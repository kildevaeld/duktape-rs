use super::context::Context;
use super::error::Result;
use duktape_sys;
use std::ffi::{CStr, CString};

pub trait Serialize {
    fn push(self, ctx: &mut Context) -> Result<()>;
}

pub trait Deserialize: Sized {
    fn get(ctx: &Context, index: i32) -> Result<Self>;
}

macro_rules! impl_for_ser {
    ($T:ty, $U:ty, $func:ident) => {
        impl Serialize for $T {
            fn push(self, context: &mut Context) -> Result<()> {
                unsafe { duktape_sys::$func(context.inner, self as $U) };
                Ok(())
            }
        }
    };
    ($T:ty, $func:ident) => {
        impl_for_ser!($T, $T, $func)
    };
}

macro_rules! impl_for_der {
    ($T:ty, $U:ty, $func:ident) => {
        impl Deserialize for $T {
            fn get(context: &Context, index: i32) -> Result<Self> {
                Ok(unsafe { duktape_sys::$func(context.inner, index) as Self })
            }
        }
    };
    ($T:ty, $func:ident) => {
        impl_for_der!($T, $T, $func)
    };
}

impl_for_ser!(isize, f64, duk_push_number);
impl_for_ser!(i8, f64, duk_push_number);
impl_for_ser!(i16, f64, duk_push_number);
impl_for_ser!(i32, f64, duk_push_number);
impl_for_ser!(usize, f64, duk_push_number);
impl_for_ser!(u8, f64, duk_push_number);
impl_for_ser!(u16, f64, duk_push_number);
impl_for_ser!(u32, f64, duk_push_number);
impl_for_ser!(bool, u32, duk_push_boolean);

impl_for_der!(isize, f64, duk_get_number);
impl_for_der!(i8, f64, duk_get_number);
impl_for_der!(i16, f64, duk_get_number);
impl_for_der!(i32, f64, duk_get_number);
impl_for_der!(usize, f64, duk_get_number);
impl_for_der!(u8, f64, duk_get_number);
impl_for_der!(u16, f64, duk_get_number);
impl_for_der!(u32, f64, duk_get_number);
//impl_for_der!(bool, u32, duk_get_boolean);

impl Serialize for String {
    fn push(self, context: &mut Context) -> Result<()> {
        let len = self.len();
        let data = CString::new(self.as_bytes()).unwrap();
        let ptr = data.as_ptr();
        unsafe {
            duktape_sys::duk_push_lstring(context.inner, ptr, len);
        };
        Ok(())
    }
}

impl<'a> Serialize for &'a String {
    fn push(self, context: &mut Context) -> Result<()> {
        let data = self.as_ptr() as *const i8;
        unsafe {
            duktape_sys::duk_push_string(context.inner, data);
        };
        Ok(())
    }
}

impl<'a> Serialize for &'a str {
    fn push(self, context: &mut Context) -> Result<()> {
        let len = self.len();
        let data = self.as_ptr() as *const i8;
        unsafe {
            duktape_sys::duk_push_lstring(context.inner, data, len);
        };
        Ok(())
    }
}

impl Deserialize for String {
    fn get(ctx: &Context, index: i32) -> Result<Self> {
        let ret = unsafe {
            let ostr = duktape_sys::duk_get_string(ctx.inner, index);
            CStr::from_ptr(ostr).to_str().unwrap().to_string()
        };
        Ok(ret)
    }
}
