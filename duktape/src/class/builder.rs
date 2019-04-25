use super::super::callable::*;
use super::super::context::{CallRet, Context, Idx, DUK_VARARGS};
use super::super::error::*;
use super::super::from_context::*;
use super::super::function::*;
use super::super::object::*;
use super::super::property::*;
use super::super::reference::{JSValue, Reference};
use super::super::to_context::*;

pub type Instance<'a> = Object<'a>;

pub trait Method {
    fn argc(&self) -> i32 {
        DUK_VARARGS
    }
    fn call(&self, ctx: &Context, instance: &Instance) -> DukResult<CallRet>;
}

impl<T: Fn(&Context, &Instance) -> DukResult<CallRet>> Method for (i32, T) {
    fn argc(&self) -> i32 {
        self.0
    }

    fn call(&self, ctx: &Context, instance: &Instance) -> DukResult<CallRet> {
        self.1(ctx, instance)
    }
}

impl<T: Fn(&Context, &Instance) -> DukResult<CallRet>> Method for T {
    fn argc(&self) -> i32 {
        0
    }

    fn call(&self, ctx: &Context, instance: &Instance) -> DukResult<CallRet> {
        self(ctx, instance)
    }
}

impl Method for Box<dyn Method> {
    fn argc(&self) -> i32 {
        self.as_ref().argc()
    }

    fn call(&self, ctx: &Context, instance: &Instance) -> DukResult<CallRet> {
        self.as_ref().call(ctx, instance)
    }
}

#[derive(Debug, Clone)]
pub struct ClassBuilder<'a> {
    ctor: Function<'a>,
}

impl<'a> ClassBuilder<'a> {
    pub fn new<M: Method + 'static>(
        ctx: &'a Context,
        ctor: M,
        parent: Option<Function>,
    ) -> DukResult<ClassBuilder<'a>> {
        let func = ctx
            .push(jsfunc((ctor.argc(), move |ctx: &Context| {
                ctx.push_this();
                let this = ctx.getp::<Object>()?;
                ctor.call(ctx, &this)
            })))?
            .getp::<Function>()?;

        if let Some(parent) = parent {
            ctx.get_global_string("Object")
                .push_string("create")
                .push(parent.clone())?;

            ctx.get_prop_string(-1, "prototype").remove(-2);
            func.prototype().unwrap().push();

            ctx.call_prop(-4, 2)?.remove(-2);

            func.set("__super__", parent)?;
        } else {
            //ctx.push_object();
            func.prototype().unwrap().push();
        }

        let proto = ctx.getp::<Object>()?;
        func.set_prototype(&proto)?;
        func.prototype().unwrap().set("constructor", func.clone())?;

        Ok(ClassBuilder { ctor: func })
    }

    pub fn method<Str: AsRef<[u8]>, M: Method + 'static>(
        &self,
        name: Str,
        method: M,
    ) -> DukResult<&Self> {
        let proto = self.ctor.prototype().unwrap();
        proto.set(
            name,
            jsfunc((method.argc(), move |ctx: &Context| {
                let this = ctx.push_this().getp::<Object>()?;
                method.call(ctx, &this)
            })),
        )?;
        Ok(self)
    }

    pub fn property<V: ToDuktape>(&self, property: PropertyBuilder<V>) -> DukResult<&Self> {
        self.ctor.prototype().unwrap().define_property(property)?;
        Ok(self)
    }
}

impl<'a> JSValue<'a> for ClassBuilder<'a> {
    fn push(&self) -> &Self {
        self.ctor.push();
        self
    }

    fn ctx(&self) -> &'a Context {
        self.ctor.ctx()
    }
}

impl<'a> JSObject<'a> for ClassBuilder<'a> {}

impl<'a> JSFunction<'a> for ClassBuilder<'a> {}

impl<'a> ToDuktape for ClassBuilder<'a> {
    fn to_context(self, _ctx: &Context) -> DukResult<()> {
        self.push();
        Ok(())
    }
}

impl<'a> FromDuktape<'a> for ClassBuilder<'a> {
    fn from_context(ctx: &'a Context, index: Idx) -> DukResult<Self> {
        let re = Reference::new(ctx, index)?;
        Ok(ClassBuilder {
            ctor: Function::new(re),
        })
    }
}
