use super::callable::{push_callable, Callable};
use super::class::{push_class_builder, Builder};
use super::error::{ErrorKind, Result};
use super::privates;
use super::types::{FromDuktape, ToDuktape};
use duktape_sys::{self as duk, duk_context};
use std::ffi::{c_void, CStr};
use std::fmt;
use std::mem;
use std::ptr;
use typemap::TypeMap;

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
    Buffer,
}

pub trait Constructable<'ctor>: Sized {
    fn construct(duk: &'ctor Context) -> Result<Self>;
}

bitflags! {
    pub struct Compile: u32 {
        const EVAL = 8;
        const FUNCTION = 16;
        const STRICT = 32;
        const SHEBANG = 64;
        const SAFE = 128;
        const NORESULT = 256;
        const NOSOURCE = 512;
        const STRLEN = 1024;
        const NOFILENAME = 2048;
        const FUNCEXPR = 4096;
    }
}

bitflags! {
    pub struct Enumerate: u32 {
        /* Enumeration flags for duk_enum() */
        const INCLUDE_NONENUMERABLE  =  (1 << 0);    /* enumerate non-numerable properties in addition to enumerable */
        const INCLUDE_HIDDEN         =  (1 << 1);    /* enumerate hidden symbols too (in Duktape 1.x called internal properties) */
        const INCLUDE_SYMBOLS        =  (1 << 2);    /* enumerate symbols */
        const EXCLUDE_STRINGS        =  (1 << 3);    /* exclude strings */
        const OWN_PROPERTIES_ONLY    =  (1 << 4);    /* don't walk prototype chain, only check own properties */
        const ARRAY_INDICES_ONLY     =  (1 << 5);    /* only enumerate array indices */
        /* XXX: misleading name */
        const SORT_ARRAY_INDICES     =  (1 << 6);    /* sort array indices (applied to full enumeration result, including inherited array indices); XXX: misleading name */
        const NO_PROXY_BEHAVIOR      = (1 << 7);    /* enumerate a proxy object itself without invoking proxy behavior */

    }
}

pub struct Context {
    pub(crate) inner: *mut duk_context,
    managed: bool,
    data: *mut TypeMap,
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
                msg = $ctx.get_string(-1)?.to_owned();
            } else {
                msg = "Uknown".to_string();
            }
            $ctx.pop(2);

            return Err(ErrorKind::TypeError(msg).into());
        }
    };
}

macro_rules! check_impl {
    ($ret:ident, $func:ident) => {
        pub fn $ret (&self, index:Idx) -> bool {
            match unsafe {
                duk::$func(self.inner, index)
            } {
                1 => true,
                _ => false,
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
    pub fn eval<T: AsRef<[u8]>>(&self, script: T) -> Result<&Self> {
        let script = script.as_ref();

        let ret = unsafe {
            duk::duk_peval_lstring(self.inner, script.as_ptr() as *const i8, script.len())
        };

        handle_error!(ret, self);

        Ok(self)
    }

    pub fn compile(&self, flags: Compile) -> Result<&Self> {
        let ret = unsafe { duk::duk_pcompile(self.inner, flags.bits()) };
        handle_error!(ret, self);

        Ok(self)
    }

    pub fn compile_string<T: AsRef<[u8]>>(&self, content: T, flags: Compile) -> Result<()> {
        let content = content.as_ref();
        let len = content.len();

        let ret = unsafe {
            duk::duk_pcompile_lstring(self.inner, flags.bits(), content.as_ptr() as *const i8, len)
        };
        handle_error!(ret, self);

        Ok(())
    }

    pub fn compile_string_filename<T: AsRef<[u8]>>(
        &self,
        content: T,
        file_name: &str,
        flags: Compile,
    ) -> Result<()> {
        let content = content.as_ref();
        let len = content.len();

        let ret = unsafe {
            duk::duk_push_lstring(self.inner, file_name.as_ptr() as *const i8, file_name.len());
            duk::duk_pcompile_lstring_filename(
                self.inner,
                flags.bits(),
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

    pub fn push_int(&self, value: i32) -> &Self {
        unsafe { duk::duk_push_int(self.inner, value) };
        self
    }

    pub fn push_uint(&self, value: u32) -> &Self {
        unsafe { duk::duk_push_uint(self.inner, value) };
        self
    }

    pub fn push_number<T: Into<f64>>(&self, value: T) -> &Self {
        unsafe { duk::duk_push_number(self.inner, value.into()) };
        self
    }

    pub fn push_string<T: AsRef<[u8]>>(&self, value: T) -> &Self {
        let len = value.as_ref().len();
        unsafe { duk::duk_push_lstring(self.inner, value.as_ref().as_ptr() as *const i8, len) };
        self
    }

    pub fn push_bytes<T: AsRef<[u8]>>(&self, value: T) -> &Self {
        let value = value.as_ref();
        let buffer =
            unsafe { duktape_sys::duk_push_fixed_buffer(self.inner, value.len()) } as *mut u8;

        unsafe { ptr::copy(value.as_ptr(), buffer, value.len()) };
        self
    }

    pub fn push_function<T: 'static + Callable>(&self, call: T) -> &Self {
        let c = Box::new(call);
        unsafe { push_callable(self, c) };
        self
    }

    push_impl!(push_object, duk_push_object);
    push_impl!(push_bare_object, duk_push_bare_object);
    push_impl!(push_array, duk_push_array);
    push_impl!(push_global_object, duk_push_global_object);
    push_impl!(push_global_stash, duk_push_global_stash);
    push_impl!(push_this, duk_push_this);
    push_impl!(push_current_function, duk_push_current_function);

    pub fn is_buffer(&self, idx: Idx) -> bool {
        unsafe {
            if duk::duk_is_buffer(self.inner, idx) == 1 {
                true
            } else if duk::duk_is_buffer_data(self.inner, idx) == 1 {
                true
            } else {
                false
            }
        }
    }

    pub fn get_number(&self, idx: Idx) -> Result<f64> {
        if !self.is_number(idx) {
            bail!(ErrorKind::TypeError(format!("number")));
        }
        let ret = unsafe { duk::duk_get_number(self.inner, idx) };
        Ok(ret)
    }

    pub fn get_int(&self, idx: Idx) -> Result<i32> {
        if !self.is_number(idx) {
            bail!(ErrorKind::TypeError(format!("number")));
        }
        let ret = unsafe { duk::duk_get_int(self.inner, idx) };
        Ok(ret)
    }

    pub fn get_uint(&self, idx: Idx) -> Result<u32> {
        if !self.is_number(idx) {
            bail!(ErrorKind::TypeError(format!("number")));
        }
        let ret = unsafe { duk::duk_get_uint(self.inner, idx) };
        Ok(ret)
    }

    pub fn get_boolean(&self, idx: Idx) -> Result<bool> {
        if !self.is_boolean(idx) {
            bail!(ErrorKind::TypeError(format!("boolean")));
        }
        let ok = unsafe { duk::duk_get_boolean(self.inner, idx) };
        Ok(if ok == 1 { true } else { false })
    }

    pub fn get_string(&self, idx: Idx) -> Result<&str> {
        if !self.is_string(idx) {
            bail!(ErrorKind::TypeError(format!("string")));
        }
        let ostr = unsafe { duk::duk_get_string(self.inner, idx) };
        let s = unsafe { CStr::from_ptr(ostr).to_str()? }; //.to_string();
        Ok(s)
    }

    pub fn get_bytes(&self, idx: Idx) -> Result<&[u8]> {
        if !self.is_buffer(idx) {
            bail!(ErrorKind::TypeError(format!("buffer")));
        }

        let r = unsafe {
            let mut len: usize = 0;
            let ptr = if duk::duk_is_buffer(self.inner, idx) == 1 {
                duk::duk_get_buffer(self.inner, idx, &mut len)
            } else {
                duk::duk_get_buffer_data(self.inner, idx, &mut len)
            };
            let r = std::slice::from_raw_parts_mut(ptr as *mut u8, len) as *mut [u8];
            //Vec::from_raw_parts(ptr: *mut T, length: usize, capacity: usize)
            &*r
        };

        Ok(r)
    }

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
        match unsafe { duk::duk_is_valid_index(self.inner, index) } {
            1 => true,
            _ => false,
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
        match unsafe {
            duk::duk_has_prop_lstring(
                self.inner,
                index,
                name.as_ref().as_ptr() as *const i8,
                name.as_ref().len(),
            )
        } {
            1 => true,
            _ => false,
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

    pub fn call(&self, args: i32) -> Result<&Self> {
        let ret = unsafe { duk::duk_pcall(self.inner, args) };
        handle_error!(ret, self);
        Ok(self)
    }

    pub fn call_method(&self, args: i32) -> Result<&Self> {
        let ret = unsafe { duk::duk_pcall_method(self.inner, args) };
        handle_error!(ret, self);
        Ok(self)
    }

    pub fn call_prop(&self, idx: Idx, args: i32) -> Result<&Self> {
        let ret = unsafe { duk::duk_pcall_prop(self.inner, idx, args) };
        handle_error!(ret, self);
        Ok(self)
    }

    pub fn construct(&self, args: i32) -> Result<&Self> {
        let ret = unsafe { duk::duk_pnew(self.inner, args) };
        handle_error!(ret, self);
        Ok(self)
    }

    pub fn set_finalizer(&self, idx: Idx) -> &Self {
        unsafe { duk::duk_set_finalizer(self.inner, idx) };
        self
    }

    pub fn instance_of(&self, this: Idx, that: Idx) -> bool {
        match unsafe { duk::duk_instanceof(self.inner, this, that) } {
            1 => true,
            _ => false,
        }
    }

    // Class
    pub fn push_class(&self, builder: Builder) -> Result<&Self> {
        let ret = unsafe { push_class_builder(self, builder) };
        match ret {
            Ok(_) => Ok(self),
            Err(e) => Err(e),
        }
    }

    pub fn create<'a, T: Constructable<'a>>(&'a self) -> Result<T> {
        T::construct(self)
    }

    pub fn push<T: ToDuktape>(&self, value: T) -> Result<&Self> {
        value.to_context(self)?;
        Ok(self)
    }

    pub fn get<'de, T: FromDuktape<'de>>(&'de self, index: Idx) -> Result<T> {
        T::from_context(self, index)
    }

    pub fn getp<'de, T: FromDuktape<'de>>(&'de self) -> Result<T> {
        let ret = T::from_context(self, -1);
        self.pop(1);
        ret
    }

    pub fn enumerator(&self, index: Idx, flags: Enumerate) -> Result<()> {
        unsafe { duk::duk_enum(self.inner, index, flags.bits()) };
        Ok(())
    }

    pub fn next(&self, enum_idx: Idx, value: bool) -> Result<bool> {
        let out = unsafe {
            match duk::duk_next(self.inner, enum_idx, if value { 1 } else { 0 }) {
                1 => true,
                _ => false,
            }
        };
        Ok(out)
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

impl std::cmp::PartialEq for Context {
    fn eq(&self, other: &Context) -> bool {
        self.inner == other.inner
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
    fn context_push_function() {
        let duk = Context::new().unwrap();
        duk.push_function(|ctx: &Context| {
            ctx.push_int(42);
            Ok(1)
        });

        duk.call(0).unwrap();
        assert_eq!(duk.get_int(-1).unwrap(), 42);
    }

    #[test]
    fn context_push_function_args() {
        let duk = Context::new().unwrap();
        duk.push_function((1, |ctx: &Context| {
            ctx.push_int(42);
            Ok(1)
        }));

        duk.call(0).unwrap();
        assert_eq!(duk.get_int(-1).unwrap(), 42);
    }

    #[test]
    fn context_push_bytes() {
        let duk = Context::new().unwrap();

        let bs = b"Hello, World";
        duk.push_bytes(bs);

        assert_eq!(duk.is_buffer(-1), true);
        let bs2 = duk.get_bytes(-1).unwrap();
        assert_eq!(bs, bs2);
    }

}
