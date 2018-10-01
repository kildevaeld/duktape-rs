use super::encoding::{Deserialize, Serialize};
use super::error::{ErrorKind, Result};
use super::internal;
use super::references;

use duktape_sys::{self as duk, duk_context};
use std::ffi::CStr;
use std::ptr;

pub struct Context {
    pub(crate) inner: *mut duk_context,
    managed: bool,
}

pub type Idx = i32;

#[derive(PartialEq)]
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

impl Context {
    /// Create a new context
    /// Will return an error, if a duk heap couldn't be created
    /// The context manage the lifetime of the wrapped duktape context
    pub fn new() -> Result<Context> {
        let d = unsafe { duk::duk_create_heap_default() };
        if d.is_null() {
            return Err(ErrorKind::InsufficientMemory.into());
        }

        unsafe { internal::init_refs(d) };

        Ok(Context {
            inner: d,
            managed: true,
        })
    }

    /// Create a new context, from a given duktape context
    /// The duktape context will **not** be managed.
    pub fn with(duk: *mut duk_context) -> Context {
        unsafe { internal::init_refs(duk) };
        Context {
            inner: duk,
            managed: false,
        }
    }

    pub fn ptr(&self) -> *mut duk_context {
        self.inner
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
        let result = self.get::<T>(-1)?;
        self.pop(1);
        Ok(result)
    }

    /// Push a value to the stack
    pub fn push<T: Serialize>(&self, value: T) -> &Self {
        value.to_context(self).unwrap();
        self
    }

    pub fn push_object(&self) -> &Self {
        unsafe {
            duk::duk_push_object(self.inner);
        }
        self
    }

    pub fn push_bare_object(&self) -> &Self {
        unsafe {
            duk::duk_push_bare_object(self.inner);
        }
        self
    }

    pub fn push_global_object(&self) -> &Self {
        unsafe {
            duk::duk_push_global_object(self.inner);
        }
        self
    }

    pub fn push_global_stash(&self) -> &Self {
        unsafe {
            duk::duk_push_global_stash(self.inner);
        }
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

    /// Checks

    pub fn is_string(&self, index: i32) -> bool {
        unsafe {
            if duk::duk_is_string(self.inner, index) == 1 {
                true
            } else {
                false
            }
        }
    }

    pub fn is_number(&self, index: i32) -> bool {
        unsafe {
            if duk::duk_is_number(self.inner, index) == 1 {
                true
            } else {
                false
            }
        }
    }

    pub fn is_boolean(&self, index: i32) -> bool {
        unsafe {
            if duk::duk_is_boolean(self.inner, index) == 1 {
                true
            } else {
                false
            }
        }
    }

    pub fn is_object(&self, index: i32) -> bool {
        unsafe {
            if duk::duk_is_object(self.inner, index) == 1 {
                true
            } else {
                false
            }
        }
    }

    pub fn is_function(&self, index: i32) -> bool {
        unsafe {
            if duk::duk_is_function(self.inner, index) == 1 {
                true
            } else {
                false
            }
        }
    }

    pub fn is_undefined(&self, index: Idx) -> bool {
        unsafe {
            if duk::duk_is_undefined(self.inner, index) == 1 {
                true
            } else {
                false
            }
        }
    }

    pub fn is_array(&self, index: Idx) -> bool {
        unsafe {
            if duk::duk_is_array(self.inner, index) == 1 {
                true
            } else {
                false
            }
        }
    }

    pub fn is(&self, t: Type) -> bool {
        self.get_type(-1) == t
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
                }
                return Type::Object;
            }
            _ => Type::Undefined,
        }
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
}

impl Drop for Context {
    fn drop(&mut self) {
        if !self.inner.is_null() && self.managed {
            unsafe {
                duk::duk_destroy_heap(self.inner);
            };
        }
        self.inner = ptr::null_mut();
    }
}

#[cfg(test)]
mod tests {

    use super::Context;

    #[test]
    fn context_new() {
        let duk = Context::new();
        assert!(duk.is_some());
    }

    #[test]
    fn context_eval() {
        let duk = Context::new().unwrap();
        duk.eval("2 + 2");
    }

}
