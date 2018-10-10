use super::encoding::{Deserialize, Serialize};
use super::error::{ErrorKind, Result};
use super::privates;
use duktape_sys::{self as duk, duk_context};
use std::ffi::CStr;
use std::fmt;
use std::ptr;
use typemap::TypeMap;

pub struct Context {
    pub(crate) inner: *mut duk_context,
    managed: bool,
    data: *mut TypeMap,
}

pub type Idx = i32;

#[derive(PartialEq, Debug)]
pub enum Type {
    Undefined,
    Null,
    String,
    Boolean,
    Number,
    Object,
    Array,
    Function,
}

pub trait Constructable<'ctor>: Sized {
    fn construct(duk: &'ctor Context) -> Result<Self>;
}

macro_rules! handle_error {
    ($ret: expr, $ctx: expr) => {
        if ($ret) != duk::DUK_EXEC_SUCCESS as i32 {
            if $ctx.has_prop_string(-1, "stack") {
                $ctx.get_prop_string(-1, "stack");
            } else {
                $ctx.get_prop_string(-1, "message");
            }

            let msg: String;
            if $ctx.is_string(-1) {
                msg = $ctx.getp()?;
            } else {
                msg = "Uknown".to_string();
                $ctx.pop(1);
            }

            $ctx.pop(1);

            return Err(ErrorKind::TypeError(msg).into());
        }
    };
}

macro_rules! check_impl {
    ($ret:ident, $func:ident) => {
        pub fn $ret (&self, index:Idx) -> bool {
            unsafe {
                if duk::$func(self.inner, index) == 1 {
                    true
                } else {
                    false
                }
            }
        }
    };
}

macro_rules! push_impl {
    ($ret:ident, $func:ident) => {
        pub fn $ret (&self) -> &Self {
            unsafe {
                duk::$func(self.inner)
            };
            self
        }
    };
}

impl Context {
    /// Create a new context
    /// Will return an error, if a duk heap couldn't be created
    /// The context manage the lifetime of the wrapped duktape context
    pub fn new() -> Result<Context> {
        let d = unsafe { duk::duk_create_heap_default() };
        if d.is_null() {
            return Err(ErrorKind::InsufficientMemory.into());
        }

        unsafe { privates::init_refs(d) };
        unsafe { privates::init_data(d) };
        Ok(Context {
            inner: d,
            managed: true,
            data: unsafe { privates::get_data(d) },
        })
    }

    /// Create a new context, from a given duktape context
    /// The duktape context will **not** be managed.
    pub(crate) fn with(duk: *mut duk_context) -> Context {
        unsafe { privates::init_refs(duk) };
        unsafe { privates::init_data(duk) };
        Context {
            inner: duk,
            managed: false,
            data: unsafe { privates::get_data(duk) },
        }
    }

    pub fn ptr(&self) -> *mut duk_context {
        self.inner
    }

    pub fn data<'a>(&'a self) -> Result<&'a TypeMap> {
        unsafe {
            if self.data.is_null() {
                return Err(ErrorKind::InsufficientMemory.into());
            }
            Ok(&*self.data)
        }
    }

    pub fn data_mut<'a>(&'a self) -> Result<&'a mut TypeMap> {
        unsafe {
            if self.data.is_null() {
                return Err(ErrorKind::InsufficientMemory.into());
            }
            Ok(&mut *self.data)
        }
    }

    /// Evaluate a script
    pub fn eval<T: AsRef<[u8]>>(&self, script: T) -> Result<()> {
        let script = script.as_ref();

        let ret = unsafe {
            duk::duk_peval_lstring(self.inner, script.as_ptr() as *const i8, script.len())
        };

        handle_error!(ret, self);

        Ok(())
    }

    pub fn compile(&self, flags: u32) -> Result<&Self> {
        let ret = unsafe { duk::duk_pcompile(self.inner, flags) };
        handle_error!(ret, self);

        Ok(self)
    }

    pub fn compile_string<T: AsRef<[u8]>>(&self, content: T, flags: u32) -> Result<()> {
        let content = content.as_ref();
        let len = content.len();

        let ret = unsafe {
            duk::duk_pcompile_lstring(self.inner, flags, content.as_ptr() as *const i8, len)
        };
        handle_error!(ret, self);

        Ok(())
    }

    pub fn compile_string_filename<T: AsRef<[u8]>>(
        &self,
        content: T,
        file_name: &str,
        flags: u32,
    ) -> Result<()> {
        let content = content.as_ref();
        let len = content.len();

        let ret = unsafe {
            duk::duk_push_lstring(self.inner, file_name.as_ptr() as *const i8, file_name.len());
            duk::duk_pcompile_lstring_filename(
                self.inner,
                flags,
                content.as_ptr() as *const i8,
                len,
            )
        };
        handle_error!(ret, self);

        Ok(())
    }

    pub fn dump(&self) -> String {
        unsafe {
            duk::duk_push_context_dump(self.inner);
            let ostr = duk::duk_get_string(self.inner, -1);
            let s = CStr::from_ptr(ostr).to_str().unwrap().to_string();
            duk::duk_pop(self.inner);
            s
        }
    }

    pub fn get<'a, T: Deserialize<'a>>(&'a self, index: Idx) -> Result<T> {
        T::from_context(self, index)
    }

    pub fn getp<'a, T: Deserialize<'a>>(&'a self) -> Result<T> {
        let result = T::from_context(self, -1)?;
        self.pop(1);
        Ok(result)
    }

    /// Push a value to the stack
    pub fn push<T: Serialize>(&self, value: T) -> &Self {
        value.to_context(self).unwrap();
        self
    }

    pub fn create<'a, T: Constructable<'a>>(&'a self) -> Result<T> {
        T::construct(self)
    }

    pub fn push_null(&self) -> &Self {
        unsafe { duk::duk_push_null(self.inner) };
        self
    }

    pub fn push_undefined(&self) -> &Self {
        unsafe { duk::duk_push_undefined(self.inner) };
        self
    }

    pub fn push_boolean(&self, value: bool) -> &Self {
        let out = if value { 1 } else { 0 };
        unsafe { duk::duk_push_boolean(self.inner, out) };
        self
    }

    push_impl!(push_object, duk_push_object);
    push_impl!(push_bare_object, duk_push_bare_object);
    push_impl!(push_array, duk_push_array);
    push_impl!(push_global_object, duk_push_global_object);
    push_impl!(push_global_stash, duk_push_global_stash);
    push_impl!(push_this, duk_push_this);
    push_impl!(push_current_function, duk_push_current_function);

    pub fn get_global_string<T: AsRef<[u8]>>(&self, name: T) -> &Self {
        unsafe {
            duk::duk_get_global_lstring(
                self.inner,
                name.as_ref().as_ptr() as *const i8,
                name.as_ref().len(),
            )
        };
        self
    }

    pub fn dup(&self, idx: Idx) -> &Self {
        unsafe { duk::duk_dup(self.inner, idx) };
        self
    }

    pub fn pop(&self, mut index: Idx) -> &Self {
        let top = self.top();
        if index > top {
            index = top;
        }
        if index == 0 {
            return self;
        }
        unsafe {
            duk::duk_pop_n(self.inner, index);
        };

        self
    }

    pub fn remove(&self, idx: Idx) -> &Self {
        unsafe { duk::duk_remove(self.inner, idx) };
        self
    }

    pub fn top(&self) -> i32 {
        unsafe { duk::duk_get_top(self.inner) }
    }

    pub fn is_valid_index(&self, index: i32) -> bool {
        unsafe {
            if duk::duk_is_valid_index(self.inner, index) == 1 {
                true
            } else {
                false
            }
        }
    }

    pub fn normalize_index(&self, idx: Idx) -> Idx {
        unsafe { duk::duk_normalize_index(self.inner, idx) }
    }

    pub fn get_length(&self, idx: Idx) -> usize {
        unsafe { duk::duk_get_length(self.inner, idx) }
    }

    /// Properties

    ///
    pub fn put_prop_string<T: AsRef<[u8]>>(&self, index: i32, name: T) -> &Self {
        unsafe {
            duk::duk_put_prop_lstring(
                self.inner,
                index,
                name.as_ref().as_ptr() as *const i8,
                name.as_ref().len(),
            );
        }
        self
    }

    pub fn get_prop_string<T: AsRef<[u8]>>(&self, index: i32, name: T) -> &Self {
        unsafe {
            duk::duk_get_prop_lstring(
                self.inner,
                index,
                name.as_ref().as_ptr() as *const i8,
                name.as_ref().len(),
            );
        }
        self
    }

    pub fn del_prop_string<T: AsRef<[u8]>>(&self, index: i32, name: T) -> &Self {
        unsafe {
            duk::duk_del_prop_lstring(
                self.inner,
                index,
                name.as_ref().as_ptr() as *const i8,
                name.as_ref().len(),
            );
        }
        self
    }

    pub fn has_prop_string<T: AsRef<[u8]>>(&self, index: i32, name: T) -> bool {
        unsafe {
            if duk::duk_has_prop_lstring(
                self.inner,
                index,
                name.as_ref().as_ptr() as *const i8,
                name.as_ref().len(),
            ) == 1
            {
                true
            } else {
                false
            }
        }
    }

    pub fn put_prop_index(&self, aidx: Idx, index: u32) -> &Self {
        unsafe {
            duk::duk_put_prop_index(self.inner, aidx, index);
        }
        self
    }

    pub fn get_prop_index(&self, aidx: Idx, index: u32) -> &Self {
        unsafe {
            duk::duk_get_prop_index(self.inner, aidx, index);
        }
        self
    }

    pub fn del_prop_index(&self, aidx: Idx, index: u32) -> &Self {
        unsafe {
            duk::duk_del_prop_index(self.inner, aidx, index);
        }
        self
    }

    pub fn has_prop_index(&self, aidx: Idx, index: u32) -> bool {
        unsafe {
            if duk::duk_has_prop_index(self.inner, aidx, index) == 1 {
                true
            } else {
                false
            }
        }
    }

    /// Checks
    /// Check if value at index is a string
    check_impl!(is_string, duk_is_string);

    check_impl!(is_number, duk_is_number);

    check_impl!(is_boolean, duk_is_boolean);

    check_impl!(is_object, duk_is_object);

    check_impl!(is_function, duk_is_function);

    check_impl!(is_undefined, duk_is_undefined);

    check_impl!(is_null, duk_is_null);

    check_impl!(is_array, duk_is_array);

    pub fn is(&self, t: Type, idx: Idx) -> bool {
        self.get_type(idx) == t
    }

    pub fn get_type(&self, index: Idx) -> Type {
        let duk_type = unsafe { duk::duk_get_type(self.inner, index) as u32 };

        match duk_type {
            duk::DUK_TYPE_UNDEFINED => Type::Undefined,
            duk::DUK_TYPE_NULL => Type::Null,
            duk::DUK_TYPE_BOOLEAN => Type::Boolean,
            duk::DUK_TYPE_NUMBER => Type::Number,
            duk::DUK_TYPE_STRING => Type::String,
            duk::DUK_TYPE_OBJECT => {
                if self.is_function(index) {
                    return Type::Function;
                } else if self.is_array(index) {
                    return Type::Array;
                }
                return Type::Object;
            }
            _ => Type::Undefined,
        }
    }

    // Strings
    pub fn concat(&self, argc: i32) -> Result<()> {
        if argc > self.top() {
            return Err(ErrorKind::ReferenceError(format!("invalid index: {}", argc)).into());
        }
        unsafe { duk::duk_concat(self.inner, argc) };
        Ok(())
    }

    pub fn call(&self, args: i32) -> Result<()> {
        let ret = unsafe { duk::duk_pcall(self.inner, args) };
        handle_error!(ret, self);
        Ok(())
    }

    pub fn call_method(&self, args: i32) -> Result<()> {
        let ret = unsafe { duk::duk_pcall_method(self.inner, args) };
        handle_error!(ret, self);
        Ok(())
    }

    pub fn call_prop(&self, idx: Idx, args: i32) -> Result<()> {
        let ret = unsafe { duk::duk_pcall_prop(self.inner, idx, args) };
        handle_error!(ret, self);
        Ok(())
    }

    pub fn construct(&self, args: i32) -> Result<()> {
        let ret = unsafe { duk::duk_pnew(self.inner, args) };
        handle_error!(ret, self);
        Ok(())
    }

    pub fn set_finalizer(&self, idx: Idx) -> &Self {
        unsafe { duk::duk_set_finalizer(self.inner, idx) };
        self
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !self.inner.is_null() && self.managed {
            unsafe {
                duk::duk_destroy_heap(self.inner);
            };
        }

        self.data = ptr::null_mut();
        self.inner = ptr::null_mut();
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.dump())
    }
}

#[cfg(test)]
pub mod tests {

    use super::Context;

    #[test]
    fn context_new() {
        let duk = Context::new();
        assert!(duk.is_ok());
    }

    #[test]
    fn context_eval() {
        let duk = Context::new().unwrap();
        duk.eval("2 + 2").unwrap();
        let i: i8 = duk.get(-1).unwrap();
        assert_eq!(i, 4);
    }

    #[test]
    fn concat() {
        let duk = Context::new().unwrap();
        duk.push("Hello").push(",").push("World").push(2);
        duk.concat(4).unwrap();
        let i: String = duk.get(-1).unwrap();
        assert_eq!(i, "Hello,World2");

        let ret = duk.concat(4);
        assert!(ret.is_err());
    }

}
