use super::argument_list::ArgumentList;
use super::context::{Context, Idx, Type};
use super::error::DukResult;
use super::from_context::*;
use super::privates::{make_ref, push_ref, unref};
use super::to_context::*;

type RefId = u32;

pub struct Reference<'a> {
    _ref: RefId,
    ctx: &'a Context,
}

impl<'a> Reference<'a> {
    pub(crate) fn new(ctx: &'a Context, idx: Idx) -> Reference<'a> {
        ctx.dup(idx);
        let refer = unsafe { make_ref(ctx.inner) };
        Reference { ctx, _ref: refer }
    }

    // pub fn ctx(&self) -> &'a Context {
    //     self.ctx
    // }

    // pub fn get_type(&self) -> Type {
    //     unsafe { push_ref(self.ctx.inner, self.refer) };
    //     let ret = self.ctx.get_type(-1);
    //     self.ctx.pop(1);
    //     ret
    // }

    // pub fn is(&self, t: Type) -> bool {
    //     self.get_type() == t
    // }

    // pub fn get<T: FromDuktape<'a>>(&self) -> Result<T> {
    //     self.push();
    //     let ret = T::from_context(self.ctx, -1);
    //     self.ctx.pop(1);
    //     ret
    // }

    // pub fn push(&self) -> &Self {
    //     unsafe { push_ref(self.ctx.inner, self._ref) };
    //     self
    // }

    // pub fn instance_of(&self, reference: &Reference) -> bool {
    //     self.push();
    //     reference.push();
    //     let ret = self.ctx.instance_of(-2, -1);
    //     self.ctx.pop(2);
    //     ret
    // }
}

impl<'a> Drop for Reference<'a> {
    fn drop(&mut self) {
        unsafe { unref(self.ctx.inner, self._ref) };
    }
}

impl<'a> Clone for Reference<'a> {
    fn clone(&self) -> Self {
        self.push();
        let r = Reference::new(self.ctx, -1);
        self.ctx.pop(1);
        r
    }
}

impl<'a> ToDuktape for Reference<'a> {
    fn to_context(self, _ctx: &Context) -> DukResult<()> {
        self.push();
        Ok(())
    }
}

impl<'a> FromDuktape<'a> for Reference<'a> {
    fn from_context(ctx: &'a Context, index: i32) -> DukResult<Self> {
        Ok(Reference::new(ctx, index))
    }
}

impl<'a> JSValue<'a> for Reference<'a> {
    fn push(&self) -> &Self {
        unsafe { push_ref(self.ctx.inner, self._ref) };
        self
    }

    fn ctx(&self) -> &'a Context {
        self.ctx
    }
}

impl<'a> ArgumentList for Reference<'a> {
    fn len(&self) -> i32 {
        1
    }

    fn push_args(self, ctx: &Context) -> DukResult<()> {
        self.to_context(ctx)
    }
}

pub trait JSValue<'a>: Sized {
    fn push(&self) -> &Self;
    fn ctx(&self) -> &'a Context;

    fn get_type(&self) -> Type {
        self.push();
        let ret = self.ctx().get_type(-1);
        self.ctx().pop(1);
        ret
    }

    fn is(&self, t: Type) -> bool {
        self.get_type() == t
    }

    fn to<T: FromDuktape<'a>>(&self) -> DukResult<T> {
        self.push();
        let ret = self.ctx().getp::<'a, T>()?;
        self.ctx().pop(1);
        Ok(ret)
    }

    fn instance_of(&self, reference: &Reference) -> bool {
        self.push();
        reference.push();
        let ret = self.ctx().instance_of(-2, -1);
        self.ctx().pop(2);
        ret
    }
}
