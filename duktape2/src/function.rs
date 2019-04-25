use super::argument_list::ArgumentList;
use super::context::{Context, Idx};
use super::error::DukResult;
use super::object::{JSObject, Object};
use super::prelude::{FromDuktape, FromDuktapeContext, ToDuktape, ToDuktapeContext};
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

    fn call_ctx<Args: ArgumentList, Ctx: ToDuktape, T: FromDuktape<'a>>(
        &self,
        this: Ctx,
        args: Args,
    ) -> DukResult<T> {
        self.push();
        this.to_context(self.ctx())?;
        let len = args.len();
        args.push_args(self.ctx())?;
        self.ctx().call_method(len)?;
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

    fn prototype(&self) -> DukResult<Object<'a>> {
        if !self.has("prototype") {
            self.push();
            unsafe {
                duk::duk_get_prototype(self.ctx().inner, -1);
            }
            self.ctx().remove(-2);

            self.set_prototype(&self.ctx().getp::<Object>()?)?;
            //return None;
        }
        // self.push();
        // unsafe {
        //     duk::duk_get_prototype(self.ctx().inner, -1);
        // }
        // self.ctx().remove(-2);
        // println!("ctx2 {:?}", self.ctx());
        // Some(self.ctx().getp::<Object>().unwrap())
        self.get("prototype")
    }

    fn set_prototype(&self, prototype: &Object<'a>) -> DukResult<&Self> {
        self.set("prototype", prototype)?;

        Ok(self)
        // self.push();
        // self.ctx().push(prototype)?;
        // unsafe {
        //     duk::duk_set_prototype(self.ctx().inner, -2);
        // }
        // self.ctx().pop(1);
        // // self.ctx().push_string("prototype").push(prototype)?;

        // // unsafe {
        // //     duk::duk_def_prop(
        // //         self.ctx().inner,
        // //         -3,
        // //         duk::DUK_DEFPROP_HAVE_VALUE | duk::DUK_DEFPROP_FORCE,
        // //     );
        // // }
        // // self.ctx().pop(1);
        // Ok(self)
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
        f.debug_struct("Function")
            .field("reference", &self._ref)
            .finish()
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
