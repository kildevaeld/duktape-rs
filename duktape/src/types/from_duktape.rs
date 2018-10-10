use super::super::{
    error::{ErrorKind, Result},
    Context, Idx,
};

pub trait FromDuktape<'de>: Sized {
    fn from_context(ctx: &'de Context, index: Idx) -> Result<Self>;
}

macro_rules! impl_for_der {
    ($T:ty, $func:ident, $check:ident) => {
        impl<'de> FromDuktape<'de> for $T {
            fn from_context(ctx: &'de Context, index: Idx) -> Result<Self> {
                if !ctx.$check(index) {
                    bail!(ErrorKind::TypeError("".to_string()));
                }
                let ret = ctx.$func(index)?;
                Ok(ret as $T)
            }
        }
    };
}

impl_for_der!(isize, get_number, is_number);
impl_for_der!(i8, get_int, is_number);
impl_for_der!(i16, get_int, is_number);
impl_for_der!(i32, get_int, is_number);
impl_for_der!(usize, get_number, is_number);
impl_for_der!(u8, get_uint, is_number);
impl_for_der!(u16, get_uint, is_number);
impl_for_der!(u32, get_uint, is_number);

impl<'de> FromDuktape<'de> for bool {
    fn from_context(ctx: &'de Context, index: Idx) -> Result<Self> {
        ctx.get_boolean(index)
    }
}

impl<'de> FromDuktape<'de> for () {
    fn from_context(_ctx: &'de Context, _index: i32) -> Result<Self> {
        Ok(())
    }
}

impl<'de> FromDuktape<'de> for String {
    fn from_context(ctx: &'de Context, index: Idx) -> Result<Self> {
        if !ctx.is_string(index) {
            return Err(ErrorKind::TypeError(format!(
                "expected string, got: {:?}",
                ctx.get_type(index)
            ))
            .into());
        }
        Ok(ctx.get_string(index)?.to_owned())
    }
}

impl<'de> FromDuktape<'de> for &'de str {
    fn from_context(ctx: &'de Context, index: Idx) -> Result<Self> {
        if !ctx.is_string(index) {
            bail!(ErrorKind::TypeError(format!(
                "expected string, got: {:?}",
                ctx.get_type(index)
            )));
        }
        ctx.get_string(index)
    }
}