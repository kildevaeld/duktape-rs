use super::super::context::{Context, Idx};
use super::super::error::Result;
use super::argument_list::ArgumentList;
use super::reference::Ref;
use super::{FromDuktape, ToDuktape};
use duktape_sys as duk;

pub struct Function<'a> {
    pub(crate) refer: Ref<'a>,
}

impl<'a> Function<'a> {
    pub(crate) fn new(refer: Ref<'a>) -> Function<'a> {
        Function { refer }
    }

    pub fn call<Args: ArgumentList, T: FromDuktape<'a>>(&mut self, args: Args) -> Result<T> {
        self.refer.push();
        let len = args.len();
        args.push_args(self.refer.ctx)?;
        self.refer.ctx.call(len)?;
        let ret = T::from_context(self.refer.ctx, -1)?;
        self.refer.ctx.pop(1);
        Ok(ret)
    }

    pub fn set_name<T: AsRef<str>>(&mut self, name: T) -> &mut Self {
        self.refer.push();
        self.refer
            .ctx
            .push_string("name")
            .push_string(name.as_ref());

        unsafe {
            duk::duk_def_prop(
                self.refer.ctx.inner,
                -3,
                duk::DUK_DEFPROP_HAVE_VALUE | duk::DUK_DEFPROP_FORCE,
            );
        }
        self.refer.ctx.pop(1);
        self
    }
}

impl<'a> ToDuktape for Function<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> ToDuktape for &'a Function<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> FromDuktape<'a> for Function<'a> {
    fn from_context(ctx: &'a Context, index: Idx) -> Result<Self> {
        let re = Ref::new(ctx, index);
        Ok(Function::new(re))
    }
}

impl<'a> Clone for Function<'a> {
    fn clone(&self) -> Self {
        self.refer.push();
        let r = Function::new(self.refer.clone());
        self.refer.ctx.pop(1);
        r
    }
}
