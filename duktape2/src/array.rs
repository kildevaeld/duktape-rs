use super::context::Context;
use super::encoding::{Deserialize, Serialize};
use super::error::{ErrorKind, Result};
use super::reference::Reference;
use std::iter;

pub struct Array<'a> {
    refer: Reference<'a>,
}

impl<'a> Array<'a> {
    pub(crate) fn new(refer: Reference<'a>) -> Array<'a> {
        Array { refer }
    }

    pub fn push<V: Serialize>(&self, value: V) -> &Self {
        self.refer.push();
        self.refer.ctx.push(value);
        self.refer.ctx.put_prop_index(-2, self.len() as u32);
        self.refer.ctx.pop(1);

        self
    }

    pub fn get<V: Deserialize<'a>>(&self, idx: u32) -> Result<V> {
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

    pub fn iter(&'a self) -> impl iter::Iterator<Item = Reference<'a>> {
        ArrayIterator::new(self)
    }
}

impl<'a> Serialize for Array<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> Deserialize<'a> for Array<'a> {
    fn from_context(ctx: &'a Context, index: i32) -> Result<Self> {
        if !ctx.is_array(index) {
            return Err(ErrorKind::TypeError("not an array".to_string()).into());
        }
        let re = Reference::new(ctx, index);

        Ok(Array::new(re))
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
