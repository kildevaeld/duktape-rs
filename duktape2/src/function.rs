use super::argument_list::ArgumentList;
use super::context::{Context, Idx};
use super::error::DukResult;
use super::object::JSObject;
use super::prelude::{FromDuktape, ToDuktape};
use super::reference::{JSValue, Reference};
use duktape_sys as duk;
use std::fmt;

pub trait JSFunction<'a>: JSObject<'a> {
    fn call<Args: ArgumentList, T: FromDuktape<'a>>(&self, args: Args) -> DukResult<T> {
        self.push();
        let len = args.len();
        args.push_args(self.ctx())?;
        self.ctx().call(len)?;
        let ret = T::from_context(self.ctx(), -1)?;
        self.ctx().pop(1);
        Ok(ret)
    }

    fn construct<Args: ArgumentList, T: FromDuktape<'a>>(&self, args: Args) -> DukResult<T> {
        self.push();
        let len = args.len();
        args.push_args(self.ctx())?;
        self.ctx().construct(len)?;
        let ret = T::from_context(self.ctx(), -1)?;
        self.ctx().pop(1);
        Ok(ret)
    }

    fn set_name<T: AsRef<str>>(&mut self, name: T) -> &mut Self {
        self.push();
        self.ctx().push_string("name").push_string(name.as_ref());

        unsafe {
            duk::duk_def_prop(
                self.ctx().inner,
                -3,
                duk::DUK_DEFPROP_HAVE_VALUE | duk::DUK_DEFPROP_FORCE,
            );
        }
        self.ctx().pop(1);
        self
    }

    fn name(&self) -> &'a str {
        self.get("name").unwrap_or("")
    }
}

#[derive(Clone)]
pub struct Function<'a> {
    pub(crate) _ref: Reference<'a>,
}

impl<'a> Function<'a> {
    pub(crate) fn new(refer: Reference<'a>) -> Function<'a> {
        Function { _ref: refer }
    }
}


impl<'a> fmt::Debug for Function<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Function").field("reference", &self._ref).finish()
    }
}


impl<'a> JSValue<'a> for Function<'a> {
    fn ctx(&self) -> &'a Context {
        self._ref.ctx()
    }
    fn push(&self) -> &Self {
        self._ref.push();
        self
    }
}

impl<'a> JSFunction<'a> for Function<'a> {}

impl<'a> JSObject<'a> for Function<'a> {}

impl<'a> ToDuktape for Function<'a> {
    fn to_context(self, _ctx: &Context) -> DukResult<()> {
        self.push();
        Ok(())
    }
}

// impl<'a> ToDuktape for &'a Function<'a> {
//     fn to_context(self, _ctx: &Context) -> DukResult<()> {
//         self.push();
//         Ok(())
//     }
// }

impl<'a> FromDuktape<'a> for Function<'a> {
    fn from_context(ctx: &'a Context, index: Idx) -> DukResult<Self> {
        let re = Reference::new(ctx, index)?;
        Ok(Function::new(re))
    }
}

impl<'a> ArgumentList for Function<'a> {
    fn len(&self) -> i32 {
        1
    }

    fn push_args(self, ctx: &Context) -> DukResult<()> {
        self.to_context(ctx)
    }
}
