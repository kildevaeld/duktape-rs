use super::super::{error::Result, Context};
use std::collections::{BTreeMap, HashMap};
use std::ptr;
#[cfg(feature = "value")]
use value::{chrono::Datelike, chrono::Timelike, Date, DateTime, Number, Value};

pub trait ToDuktape {
    fn to_context(self, ctx: &Context) -> Result<()>;
}

macro_rules! impl_for_ser {
    ($T:ty, $U:ty, $func:ident) => {
        impl ToDuktape for $T {
            fn to_context(self, ctx: &Context) -> Result<()> {
                ctx.$func(self as $U);
                Ok(())
            }
        }

        impl ToDuktape for &$T {
            fn to_context(self, ctx: &Context) -> Result<()> {
                ctx.$func(*self as $U);
                Ok(())
            }
        }
    };
}

impl_for_ser!(isize, f64, push_number);
impl_for_ser!(i8, i32, push_int);
impl_for_ser!(i16, i32, push_int);
impl_for_ser!(i32, i32, push_int);
impl_for_ser!(i64, f64, push_number);
impl_for_ser!(u64, f64, push_number);
impl_for_ser!(usize, f64, push_number);
impl_for_ser!(u8, u32, push_uint);
impl_for_ser!(u16, u32, push_uint);
impl_for_ser!(u32, u32, push_uint);
impl_for_ser!(f64, f64, push_number);
impl_for_ser!(bool, bool, push_boolean);

impl ToDuktape for () {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.push_undefined();
        Ok(())
    }
}

impl ToDuktape for &() {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.push_undefined();
        Ok(())
    }
}

impl<T: ToDuktape> ToDuktape for Option<T> {
    fn to_context(self, ctx: &Context) -> Result<()> {
        match self {
            Some(t) => t.to_context(ctx)?,
            None => {
                ctx.push_null();
            }
        };
        Ok(())
    }
}

impl ToDuktape for String {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.push_string(self);
        Ok(())
    }
}

impl<'a> ToDuktape for &'a str {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.push_string(self);
        Ok(())
    }
}

impl<'a> ToDuktape for &'a &'a str {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.push_string(self);
        Ok(())
    }
}

impl<'a> ToDuktape for &'a String {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.push_string(self);
        Ok(())
    }
}

impl<'a, T: ToDuktape> ToDuktape for Vec<T> {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.push_array();
        let mut i = 0;
        for v in self {
            v.to_context(ctx)?;
            ctx.put_prop_index(-2, i);
            i += 1;
        }
        Ok(())
    }
}

impl<T: ToDuktape> ToDuktape for HashMap<String, T> {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.push_object();
        for (k, v) in self {
            v.to_context(ctx)?;
            ctx.put_prop_string(-2, k);
        }
        Ok(())
    }
}

impl<T: ToDuktape> ToDuktape for BTreeMap<String, T> {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.push_object();
        for (k, v) in self {
            v.to_context(ctx)?;
            ctx.put_prop_string(-2, k);
        }
        Ok(())
    }
}

impl ToDuktape for &[u8] {
    fn to_context(self, ctx: &Context) -> Result<()> {
        let buffer =
            unsafe { duktape_sys::duk_push_fixed_buffer(ctx.inner, self.len()) } as *mut u8;

        unsafe { ptr::copy(self.as_ptr(), buffer, self.len()) };
        Ok(())
    }
}

#[cfg(feature = "value")]
impl ToDuktape for DateTime {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.get_global_string("Date")
            .get_prop_string(-1, "UTC")
            .remove(-2)
            .push(self.year())?
            .push(self.month())?
            .push(self.day())?
            .push(self.hour())?
            .push(self.minute())?
            .push(self.nanosecond())?
            .construct(6)?;

        Ok(())
    }
}

#[cfg(feature = "value")]
impl ToDuktape for Date {
    fn to_context(self, ctx: &Context) -> Result<()> {
        ctx.get_global_string("Date")
            .get_prop_string(-1, "UTC")
            .remove(-2)
            .push(self.year())?
            .push(self.month())?
            .push(self.day())?
            .construct(3)?;

        Ok(())
    }
}

#[cfg(feature = "value")]
impl ToDuktape for Number {
    fn to_context(self, ctx: &Context) -> Result<()> {
        if self.is_f64() {
            ctx.push_number(self.as_f64().unwrap());
        } else if self.is_u64() {
            ctx.push_uint(self.as_u64().unwrap() as u32);
        } else {
            ctx.push_int(self.as_i64().unwrap() as i32);
        }
        Ok(())
    }
}

#[cfg(feature = "value")]
impl ToDuktape for Value {
    fn to_context(self, ctx: &Context) -> Result<()> {
        match self {
            Value::Null => ctx.push_null(),
            Value::Number(n) => ctx.push(n)?,
            Value::String(n) => ctx.push_string(n),
            Value::Bool(n) => ctx.push_boolean(n),
            Value::Bytes(b) => ctx.push_bytes(b),
            Value::Array(a) => ctx.push(a)?,
            Value::Object(o) => ctx.push(o)?,
            Value::Date(n) => ctx.push(n)?,
            Value::DateTime(n) => ctx.push(n)?,
        };
        Ok(())
    }
}
