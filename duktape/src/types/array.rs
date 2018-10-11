use super::super::context::{Constructable, Context};
use super::super::error::{ErrorKind, Result};
use super::reference::Ref;
use super::{ArgumentList, FromDuktape, ToDuktape};
use std::iter;

pub struct Array<'a> {
    pub(crate) refer: Ref<'a>,
}

impl<'a> Array<'a> {
    pub(crate) fn new(refer: Ref<'a>) -> Array<'a> {
        Array { refer }
    }

    pub fn push<V: ToDuktape>(&self, value: V) -> Result<&Self> {
        self.refer.push();
        value.to_context(self.refer.ctx)?;
        self.refer.ctx.put_prop_index(-2, self.len() as u32);
        self.refer.ctx.pop(1);
        Ok(self)
    }

    pub fn get<V: FromDuktape<'a>>(&self, idx: u32) -> Result<V> {
        self.refer.push();
        self.refer.ctx.get_prop_index(-1, idx);

        let ret = self.refer.ctx.get::<V>(-1)?;

        self.refer.ctx.pop(2);

        Ok(ret)
    }

    pub fn len(&self) -> usize {
        self.refer.push();
        let ret = self.refer.ctx.get_length(-1);
        self.refer.ctx.pop(1);
        ret
    }

    pub fn iter(&'a self) -> impl iter::Iterator<Item = Ref<'a>> {
        ArrayIterator::new(self)
    }
}

impl<'a> ToDuktape for Array<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> FromDuktape<'a> for Array<'a> {
    fn from_context(ctx: &'a Context, index: i32) -> Result<Self> {
        if !ctx.is_array(index) {
            return Err(ErrorKind::TypeError("not an array".to_string()).into());
        }
        let re = Ref::new(ctx, index);

        Ok(Array::new(re))
    }
}

impl<'a> Constructable<'a> for Array<'a> {
    fn construct(duk: &'a Context) -> Result<Self> {
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

impl<'a> ArgumentList for Array<'a> {
    fn len(&self) -> i32 {
        1
    }

    fn push_args(self, ctx: &Context) -> Result<()> {
        self.to_context(ctx)
    }
}

struct ArrayIterator<'a> {
    array: &'a Array<'a>,
    index: u32,
}

impl<'a> ArrayIterator<'a> {
    pub fn new(array: &'a Array<'a>) -> ArrayIterator<'a> {
        ArrayIterator {
            array: array,
            index: 0,
        }
    }
}

impl<'a> iter::Iterator for ArrayIterator<'a> {
    type Item = Ref<'a>;

    fn next(&mut self) -> Option<Ref<'a>> {
        if self.index == self.array.len() as u32 {
            return None;
        }

        let r = match self.array.get::<Ref>(self.index) {
            Ok(m) => m,
            Err(_) => return None,
        };

        self.index += 1;

        Some(r)
    }
}
