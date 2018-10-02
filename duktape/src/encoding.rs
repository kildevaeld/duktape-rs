use super::context::Context;
use super::error::{ErrorKind, Result};
use duktape_sys;
use std::ffi::{CStr, CString};

pub trait Serialize {
    fn to_context(self, ctx: &Context) -> Result<()>;
}

pub trait Deserialize<'de>: Sized {
    fn from_context(ctx: &'de Context, index: i32) -> Result<Self>;
}

macro_rules! impl_for_ser {
    ($T:ty, $U:ty, $func:ident) => {
        impl Serialize for $T {
            fn to_context(self, context: &Context) -> Result<()> {
                unsafe { duktape_sys::$func(context.inner, self as $U) };
                Ok(())
            }
        }

        impl Serialize for &$T {
            fn to_context(self, context: &Context) -> Result<()> {
                unsafe { duktape_sys::$func(context.inner, *self as $U) };
                Ok(())
            }
        }
    };
}

macro_rules! impl_for_der {
    ($T:ty, $func:ident, $check:ident) => {
        impl<'de> Deserialize<'de> for $T {
            fn from_context(context: &'de Context, index: i32) -> Result<Self> {
                let can = unsafe {
                    if duktape_sys::$check(context.inner, index) == 1 {
                        true
                    } else {
                        false
                    }
                };
                if !can {
                    return Err(ErrorKind::TypeError("".to_string()).into());
                }
                Ok(unsafe { duktape_sys::$func(context.inner, index) as Self })
            }
        }
    };
}

impl_for_ser!(isize, f64, duk_push_number);
impl_for_ser!(i8, f64, duk_push_number);
impl_for_ser!(i16, f64, duk_push_number);
impl_for_ser!(i32, f64, duk_push_number);
impl_for_ser!(i64, f64, duk_push_number);
impl_for_ser!(usize, f64, duk_push_number);
impl_for_ser!(u8, f64, duk_push_number);
impl_for_ser!(u16, f64, duk_push_number);
impl_for_ser!(u32, f64, duk_push_number);
impl_for_ser!(bool, u32, duk_push_boolean);

impl_for_der!(isize, duk_get_number, duk_is_number);
impl_for_der!(i8, duk_get_number, duk_is_number);
impl_for_der!(i16, duk_get_number, duk_is_number);
impl_for_der!(i32, duk_get_number, duk_is_number);
impl_for_der!(usize, duk_get_number, duk_is_number);
impl_for_der!(u8, duk_get_number, duk_is_number);
impl_for_der!(u16, duk_get_number, duk_is_number);
impl_for_der!(u32, duk_get_number, duk_is_number);

impl<'de> Deserialize<'de> for bool {
    fn from_context(ctx: &'de Context, index: i32) -> Result<Self> {
        let ret = unsafe {
            let b = duktape_sys::duk_get_boolean(ctx.inner, index);
            if b == 1 {
                true
            } else {
                false
            }
        };
        Ok(ret)
    }
}

impl Serialize for () {
    fn to_context(self, context: &Context) -> Result<()> {
        unsafe { duktape_sys::duk_push_undefined(context.inner) };
        Ok(())
    }
}

impl<'de> Deserialize<'de> for () {
    fn from_context(ctx: &'de Context, index: i32) -> Result<Self> {
        // let ret = unsafe {
        //     let ostr = duktape_sys::duk_get_string(ctx.inner, index);
        //     CStr::from_ptr(ostr).to_str().unwrap().to_string()
        // };
        Ok(())
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn to_context(self, context: &Context) -> Result<()> {
        match self {
            Some(t) => context.push(t),
            None => context.push(()),
        };
        Ok(())
    }
}

impl Serialize for String {
    fn to_context(self, context: &Context) -> Result<()> {
        let len = self.len();
        let data = CString::new(self.as_bytes()).unwrap();
        let ptr = data.as_ptr();
        unsafe {
            duktape_sys::duk_push_lstring(context.inner, ptr, len);
        };
        Ok(())
    }
}

impl<'a> Serialize for &'a str {
    fn to_context(self, context: &Context) -> Result<()> {
        let len = self.len();
        let data = self.as_ptr() as *const i8;
        unsafe {
            duktape_sys::duk_push_lstring(context.inner, data, len);
        };
        Ok(())
    }
}

impl<'a> Serialize for &'a &'a str {
    fn to_context(self, context: &Context) -> Result<()> {
        let len = self.len();
        let data = self.as_ptr() as *const i8;
        unsafe {
            duktape_sys::duk_push_lstring(context.inner, data, len);
        };
        Ok(())
    }
}

impl<'a> Serialize for &'a String {
    fn to_context(self, context: &Context) -> Result<()> {
        let data = self.as_ptr() as *const i8;
        unsafe {
            duktape_sys::duk_push_string(context.inner, data);
        };
        Ok(())
    }
}

impl<'de> Deserialize<'de> for String {
    fn from_context(ctx: &'de Context, index: i32) -> Result<Self> {
        let ret = unsafe {
            if !ctx.is_string(index) {
                return Err(ErrorKind::TypeError(format!(
                    "expected string, got: {:?}",
                    ctx.get_type(index)
                ))
                .into());
            }
            let ostr = duktape_sys::duk_get_string(ctx.inner, index);
            CStr::from_ptr(ostr).to_str().unwrap().to_string()
        };
        Ok(ret)
    }
}

impl<'a, T: Serialize> Serialize for Vec<T> {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.push_array();
        let mut i = 0;
        for v in self {
            ctx.push(v).put_prop_index(-2, i);
            i += 1;
        }

        Ok(())
    }
}
