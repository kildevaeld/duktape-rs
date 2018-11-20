use super::super::{
    error::{ErrorKind, Result},
    Context, Idx,
};
#[cfg(feature = "value-rs")]
use value::{Map, Number, Value};

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

impl<'de> FromDuktape<'de> for &'de [u8] {
    fn from_context(ctx: &'de Context, index: Idx) -> Result<Self> {
        if !ctx.is_buffer(index) {
            bail!(ErrorKind::TypeError(format!(
                "expected string, got: {:?}",
                ctx.get_type(index)
            )));
        }
        ctx.get_bytes(index)
    }
}

// #[cfg(feature = "value-rs")]
// impl<'de> FromDuktape<'de> for Number {
//     fn from_context(ctx: &'de Context, index: Idx) -> Result<Self> {

//     }
// }

#[cfg(feature = "value-rs")]
impl<'de> FromDuktape<'de> for Value {
    fn from_context(ctx: &'de Context, idx: Idx) -> Result<Self> {
        let ty = ctx.get_type(idx);

        let val = match ty {
            Type::Null | Type::Undefined => Value::Null,
            Type::String => Value::String(ctx.get::<String>(idx)?),
            Type::Boolean => Value::Bool(ctx.get::<bool>(idx)?),
            Type::Number => Value::Number(Number::from_f64(ctx.get_number(idx)?)),
            Type::Object => pull_object(ctx, idx)?,
            Type::Array => pull_array(ctx, idx)?,
            _ => bail!(ErrorKind::TypeError(format!(
                "type to value not implemented: {:?}",
                ty
            ))),
        };

        Ok(val)
    }
}

#[cfg(feature = "value-rs")]
#[inline]
fn pull_object(ctx: &Context, idx: Idx) -> Result<Value> {
    ctx.enumerator(idx, Enumerate::OWN_PROPERTIES_ONLY)?;
    let mut map = Map::new();
    while ctx.next(-1, true)? {
        let key = ctx.get_string(-2)?;
        let value = Value::from_context(ctx, -1)?;
        map.insert(key.to_owned(), value);
        ctx.pop(2);
    }

    ctx.pop(1);
    Ok(Value::Object(map))
}

#[cfg(feature = "value-rs")]
#[inline]
fn pull_array(ctx: &Context, idx: Idx) -> Result<Value> {
    ctx.enumerator(idx, Enumerate::ARRAY_INDICES_ONLY)?;
    let mut map = Vec::new();
    while ctx.next(-1, true)? {
        let value = Value::from_context(ctx, -1)?;
        map.push(value);
        ctx.pop(2);
    }

    ctx.pop(1);
    Ok(Value::Array(map))
}
