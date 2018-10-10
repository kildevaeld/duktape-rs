use super::argument_list::ArgumentList;
use super::context::{Context, Idx};
use super::duktape_sys as duk;
use super::encoding::{Deserialize, Serialize};
use super::error::Result;
use super::reference::Reference;

pub struct Function<'a> {
    pub(crate) refer: Reference<'a>,
}

impl<'a> Function<'a> {
    pub(crate) fn new(refer: Reference<'a>) -> Function<'a> {
        Function { refer }
    }

    pub fn call<Args: ArgumentList, T: Deserialize<'a>>(&mut self, args: Args) -> Result<T> {
        self.refer.push();
        let len = args.len();
        args.push(self.refer.ctx);
        self.refer.ctx.call(len)?;
        let ret = self.refer.ctx.getp()?;
        Ok(ret)
    }

    pub fn set_name<T: AsRef<str>>(&mut self, name: T) -> &mut Self {
        self.refer.push();
        self.refer.ctx.push("name").push(name.as_ref());
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

impl<'a> Serialize for Function<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> Serialize for &'a Function<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> Deserialize<'a> for Function<'a> {
    fn from_context(ctx: &'a Context, index: Idx) -> Result<Self> {
        let re = Reference::new(ctx, index);
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
