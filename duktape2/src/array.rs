// use super::argument_list::ArgumentList;
use super::context::Context;
use super::error::DukResult;
use super::prelude::{Constructable, FromDuktape, FromDuktapeContext, ToDuktape, ToDuktapeContext};
use super::reference::{JSValue, Reference};
// use std::iter;

pub trait JSArray<'a>: JSValue<'a> {
    fn append<V: ToDuktape>(&self, value: V) -> DukResult<&Self> {
        self.push();
        self.ctx()
            .push(value)?
            .put_prop_index(-2, self.len() as u32)
            .pop(1);
        Ok(self)
    }

    fn get<V: FromDuktape<'a>>(&self, idx: u32) -> DukResult<V> {
        self.push();
        self.ctx().get_prop_index(-1, idx);
        let ret = self.ctx().get::<V>(-1)?;
        self.ctx().pop(2);
        Ok(ret)
    }

    fn len(&self) -> usize {
        self.push();
        let ret = self.ctx().get_length(-1);
        self.ctx().pop(1);
        ret
    }

    fn iter(&'a self) -> ArrayIterator<'a, Self> {
        ArrayIterator::new(self)
    }
}

#[derive(Clone)]
pub struct Array<'a> {
    pub(crate) _ref: Reference<'a>,
}

impl<'a> Array<'a> {
    pub(crate) fn new(refer: Reference<'a>) -> Array<'a> {
        Array { _ref: refer }
    }
}

impl<'a> JSValue<'a> for Array<'a> {
    fn push(&self) -> &Self {
        self._ref.push();
        self
    }

    fn ctx(&self) -> &'a Context {
        self._ref.ctx()
    }
}

impl<'a> JSArray<'a> for Array<'a> {}

impl<'a> Constructable<'a> for Array<'a> {
    fn construct(duk: &'a Context) -> DukResult<Self> {
        duk.push_array();
        let o = match Array::from_context(duk, -1) {
            Ok(o) => o,
            Err(e) => {
                duk.pop(1);
                return Err(e);
            }
        };
        duk.pop(1);
        Ok(o)
    }
}

impl<'a> ToDuktape for Array<'a> {
    fn to_context(self, _ctx: &Context) -> DukResult<()> {
        self.push();
        Ok(())
    }
}

impl<'a> FromDuktape<'a> for Array<'a> {
    fn from_context(ctx: &'a Context, index: i32) -> DukResult<Self> {
        if !ctx.is_array(index) {
            return duk_type_error!("not an array");
        }
        let re = Reference::new(ctx, index)?;
        Ok(Array::new(re))
    }
}

// impl<'a> ArgumentList for Array<'a> {
//     fn len(&self) -> i32 {
//         1
//     }

//     fn push_args(self, ctx: &Context) -> DukResult<()> {
//         self.to_context(ctx)
//     }
// }

pub struct ArrayIterator<'a, A: JSArray<'a>> {
    array: &'a A,
    index: u32,
}

impl<'a, A: JSArray<'a>> ArrayIterator<'a, A> {
    pub fn new(array: &'a A) -> ArrayIterator<'a, A> {
        ArrayIterator {
            array: array,
            index: 0,
        }
    }
}

impl<'a, A: JSArray<'a>> std::iter::Iterator for ArrayIterator<'a, A> {
    type Item = Reference<'a>;

    fn next(&mut self) -> Option<Reference<'a>> {
        if self.index == self.array.len() as u32 {
            return None;
        }

        let r = match self.array.get::<Reference>(self.index) {
            Ok(m) => m,
            Err(_) => return None,
        };

        self.index += 1;

        Some(r)
    }
}

impl<'a, A: JSArray<'a>> std::iter::ExactSizeIterator for ArrayIterator<'a, A> {}
