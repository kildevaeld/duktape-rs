use super::context::Context;
use super::error::DukResult;
use std::collections::{BTreeMap, HashMap};
use std::ptr;

pub trait ToDuktape {
    fn to_context(&self, ctx: &Context) -> DukResult<()>;
}

pub trait ToDuktapeContext: Sized {
    fn push<T: ToDuktape>(&self, value: T) -> DukResult<&Self>;
}

impl ToDuktapeContext for Context {
    fn push<T: ToDuktape>(&self, value: T) -> DukResult<&Self> {
        value.to_context(self)?;
        Ok(&self)
    }
}

impl<'a, T> ToDuktape for &'a T
where
    T: ToDuktape,
{
    fn to_context(&self, ctx: &Context) -> DukResult<()> {
        (*self).to_context(ctx)
    }
}

macro_rules! impl_for_ser {
    ($T:ty, $U:ty, $func:ident) => {
        impl ToDuktape for $T {
            fn to_context(&self, ctx: &Context) -> DukResult<()> {
                ctx.$func(*self as $U);
                Ok(())
            }
        }

        // impl ToDuktape for &$T {
        //     fn to_context(&self, ctx: &Context) -> DukResult<()> {
        //         ctx.$func(*self as $U);
        //         Ok(())
        //     }
        // }
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
    fn to_context(&self, ctx: &Context) -> DukResult<()> {
        ctx.push_undefined();
        Ok(())
    }
}

impl<T: ToDuktape> ToDuktape for Option<T> {
    fn to_context(&self, ctx: &Context) -> DukResult<()> {
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
    fn to_context(&self, ctx: &Context) -> DukResult<()> {
        ctx.push_string(self);
        Ok(())
    }
}

impl<'a> ToDuktape for &'a str {
    fn to_context(&self, ctx: &Context) -> DukResult<()> {
        ctx.push_string(self);
        Ok(())
    }
}

// impl<'a> ToDuktape for &'a &'a str {
//     fn to_context(&self, ctx: &Context) -> DukResult<()> {
//         ctx.push_string(self);
//         Ok(())
//     }
// }

// impl<'a> ToDuktape for &'a String {
//     fn to_context(&self, ctx: &Context) -> DukResult<()> {
//         ctx.push_string(self);
//         Ok(())
//     }
// }

impl<'a, T: ToDuktape> ToDuktape for Vec<T> {
    fn to_context(&self, ctx: &Context) -> DukResult<()> {
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
    fn to_context(&self, ctx: &Context) -> DukResult<()> {
        ctx.push_object();
        for (k, v) in self {
            v.to_context(ctx)?;
            ctx.put_prop_string(-2, k);
        }
        Ok(())
    }
}

impl<T: ToDuktape> ToDuktape for BTreeMap<String, T> {
    fn to_context(&self, ctx: &Context) -> DukResult<()> {
        ctx.push_object();
        for (k, v) in self {
            v.to_context(ctx)?;
            ctx.put_prop_string(-2, k);
        }
        Ok(())
    }
}

impl ToDuktape for &[u8] {
    fn to_context(&self, ctx: &Context) -> DukResult<()> {
        let buffer =
            unsafe { duktape_sys::duk_push_fixed_buffer(ctx.inner, self.len()) } as *mut u8;

        unsafe { ptr::copy(self.as_ptr(), buffer, self.len()) };
        Ok(())
    }
}
