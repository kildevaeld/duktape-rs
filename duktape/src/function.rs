use super::argument_list::ArgumentList;
use super::context::{Context, Idx};
use super::encoding::{Deserialize, Serialize};
use super::error::Result;
use super::reference::Reference;

pub struct Function<'a> {
    refer: Reference<'a>,
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
}

impl<'a> Serialize for Function<'a> {
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
