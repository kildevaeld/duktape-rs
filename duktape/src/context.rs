use super::encoding::{Deserialize, Serialize};
use super::error::Result;
use duktape_sys::{self as duk, duk_context};
use std::ffi::CStr;
use std::ptr;

pub struct Context {
    pub(crate) inner: *mut duk_context,
    managed: bool,
}

impl Context {
    pub fn new() -> Option<Context> {
        let d = unsafe { duk::duk_create_heap_default() };
        if d.is_null() {
            return None;
        }
        Some(Context {
            inner: d,
            managed: true,
        })
    }

    pub fn with(duk: *mut duk_context) -> Context {
        Context {
            inner: duk,
            managed: false,
        }
    }

    pub fn eval<T: AsRef<str>>(&mut self, script: T) -> Result<()> {
        let script = script.as_ref();

        let ret = unsafe {
            duk::duk_peval_lstring(self.inner, script.as_ptr() as *const i8, script.len())
        };

        if ret != duk::DUK_EXEC_SUCCESS as i32 {
            println!("{}", self.dump());
        }

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

    pub fn get<T: Deserialize>(&self, index: i32) -> Result<T> {
        T::get(self, index)
    }

    pub fn getp<T: Deserialize>(&mut self) -> Result<T> {
        let result = self.get::<T>(-1)?;
        self.pop(1);
        Ok(result)
    }

    pub fn push<T: Serialize>(&mut self, value: T) -> Result<()> {
        value.push(self)?;
        Ok(())
    }

    pub fn push_object(&mut self) {
        unsafe {
            duk::duk_push_object(self.inner);
        }
    }

    pub fn push_global_object(&mut self) {
        unsafe {
            duk::duk_push_global_object(self.inner);
        }
    }

    pub fn pop(&mut self, mut index: i32) {
        let top = self.top();
        if index > top {
            index = top;
        }
        if index == 0 {
            return;
        }
        unsafe {
            duk::duk_pop_n(self.inner, index);
        };
    }

    pub fn top(&self) -> i32 {
        unsafe { duk::duk_get_top(self.inner) }
    }

    pub fn put_prop_string<T: AsRef<str>>(&mut self, index: i32, name: T) {
        unsafe {
            duk::duk_put_prop_lstring(
                self.inner,
                index,
                name.as_ref().as_ptr() as *const i8,
                name.as_ref().len(),
            );
        }
    }

    pub fn get_prop_string<T: AsRef<str>>(&mut self, index: i32, name: T) {
        unsafe {
            duk::duk_get_prop_lstring(
                self.inner,
                index,
                name.as_ref().as_ptr() as *const i8,
                name.as_ref().len(),
            );
        }
    }

    pub fn del_prop_string<T: AsRef<str>>(&mut self, index: i32, name: T) {
        unsafe {
            duk::duk_del_prop_lstring(
                self.inner,
                index,
                name.as_ref().as_ptr() as *const i8,
                name.as_ref().len(),
            );
        }
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

// #[cfg(test)]
// mod tests {

//     use super::Context;

//     #[test]
//     fn context_new() {
//         let duk = Context::new();
//         assert!(duk.is_some());
//     }

//     #[test]
//     fn context_eval() {
//         let duk = Context::new().unwrap();
//         duk.eval("2 + 2");
//     }

// }
