use super::context::{Context, Idx};
use super::error::{DukError, DukResult};

pub trait FromDuktape<'de>: Sized {
    fn from_context(ctx: &'de Context, index: Idx) -> DukResult<Self>;
}

pub trait FromDuktapeContext: Sized {
    fn get<'de, T: FromDuktape<'de>>(&'de self, index: Idx) -> DukResult<T>;
    fn getp<'de, T: FromDuktape<'de>>(&'de self) -> DukResult<T>;
}

impl FromDuktapeContext for Context {
    fn get<'de, T: FromDuktape<'de>>(&'de self, index: Idx) -> DukResult<T> {
        T::from_context(self, index)
    }

    fn getp<'de, T: FromDuktape<'de>>(&'de self) -> DukResult<T> {
        let ret = T::from_context(self, -1);
        self.pop(1);
        ret
    }
}

macro_rules! impl_for_der {
    ($T:ty, $func:ident, $check:ident) => {
        impl<'de> FromDuktape<'de> for $T {
            fn from_context(ctx: &'de Context, index: Idx) -> DukResult<Self> {
                if !ctx.$check(index) {
                    return Err(DukError::type_err(""));
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
    fn from_context(ctx: &'de Context, index: Idx) -> DukResult<Self> {
        ctx.get_boolean(index)
    }
}

impl<'de> FromDuktape<'de> for () {
    fn from_context(_ctx: &'de Context, _index: i32) -> DukResult<Self> {
        Ok(())
    }
}

impl<'de> FromDuktape<'de> for String {
    fn from_context(ctx: &'de Context, index: Idx) -> DukResult<Self> {
        Ok(ctx.get_string(index)?.to_owned())
    }
}

impl<'de> FromDuktape<'de> for &'de str {
    fn from_context(ctx: &'de Context, index: Idx) -> DukResult<Self> {
        ctx.get_string(index)
    }
}

impl<'de> FromDuktape<'de> for &'de [u8] {
    fn from_context(ctx: &'de Context, index: Idx) -> DukResult<Self> {
        ctx.get_bytes(index)
    }
}
