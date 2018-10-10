use super::super::context::{Constructable, Context, Idx, Type};
use super::super::error::{ErrorKind, Result};
use super::argument_list::ArgumentList;
use super::function::Function;
use super::reference::Ref;
use super::{FromDuktape, ToDuktape};
use std::convert::From;
use std::fmt;

pub struct Object<'a> {
    refer: Ref<'a>,
}

impl<'a> Object<'a> {
    pub(crate) fn new(refer: Ref<'a>) -> Object<'a> {
        Object { refer }
    }

    pub fn get<T: AsRef<[u8]>, V: FromDuktape<'a>>(&self, prop: T) -> Result<V> {
        self.refer.push();
        self.refer.ctx.get_prop_string(-1, prop);
        let ret = V::from_context(self.refer.ctx, -1)?;
        self.refer.ctx.pop(2);
        Ok(ret)
    }

    pub fn set<T: AsRef<[u8]>, V: ToDuktape>(&self, prop: T, value: V) -> &Self {
        self.refer.push();
        value.to_context(self.refer.ctx).unwrap();
        self.refer.ctx.put_prop_string(-2, prop);
        self.refer.ctx.pop(1);
        self
    }

    pub fn has<T: AsRef<[u8]>>(&self, prop: T) -> bool {
        self.refer.push();
        let ret = self.refer.ctx.has_prop_string(-1, prop);
        self.refer.ctx.pop(1);
        ret
    }

    pub fn del<T: AsRef<[u8]>>(&mut self, prop: T) -> &mut Self {
        self.refer.push();
        self.refer.ctx.del_prop_string(-1, prop.as_ref());
        self.refer.ctx.pop(1);
        self
    }

    pub fn as_ref(&self) -> &'a Ref {
        &self.refer
    }

    pub fn call<T: AsRef<str>, A: ArgumentList, R: FromDuktape<'a>>(
        &self,
        fn_name: T,
        args: A,
    ) -> Result<R> {
        self.refer.push();
        let idx = self.refer.ctx.normalize_index(-1);
        self.refer.ctx.push_string(fn_name.as_ref());
        let len = args.len();
        args.push_args(self.refer.ctx)?;
        if let Err(e) = self.refer.ctx.call_prop(idx, len) {
            self.refer.ctx.pop(1);
            return Err(e);
        }
        self.refer.ctx.remove(-2);
        let ret = R::from_context(self.refer.ctx, -1);
        self.refer.ctx.pop(1);
        ret
    }

    pub fn construct<T: AsRef<str>, A: ArgumentList>(&self, fn_name: T, args: A) -> Result<Object> {
        self.refer.push();
        if !self.refer.ctx.has_prop_string(-1, fn_name.as_ref()) {
            return Err(ErrorKind::TypeError("not a function".to_owned()).into());
        }

        let len = args.len();

        self.refer.ctx.get_prop_string(-1, fn_name.as_ref());
        args.push_args(self.refer.ctx)?;
        if let Err(e) = self.refer.ctx.construct(len) {
            self.refer.ctx.pop(2);
            return Err(e);
        }

        self.refer.ctx.remove(-2);
        let o = Object::from_context(self.refer.ctx, -1);
        self.refer.ctx.pop(1);
        o
    }
}

impl<'a> ToDuktape for Object<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> ToDuktape for &'a Object<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> FromDuktape<'a> for Object<'a> {
    fn from_context(ctx: &'a Context, index: Idx) -> Result<Self> {
        let re = Ref::new(ctx, index);
        Ok(Object::new(re))
    }
}

impl<'a> Clone for Object<'a> {
    fn clone(&self) -> Self {
        Object::new(self.refer.clone())
    }
}

impl<'a> Constructable<'a> for Object<'a> {
    fn construct(duk: &'a Context) -> Result<Self> {
        duk.push_object();
        let o = match Object::from_context(duk, -1) {
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

impl<'a> From<Function<'a>> for Object<'a> {
    fn from(func: Function<'a>) -> Self {
        Object::new(func.refer.clone())
    }
}

impl<'a> From<Object<'a>> for Result<Function<'a>> {
    fn from(func: Object<'a>) -> Self {
        if func.as_ref().is(Type::Function) {
            return Ok(Function::new(func.refer.clone()));
        }
        Err(ErrorKind::TypeError("could not interpret object as function".to_owned()).into())
    }
}

impl<'a> ArgumentList for Object<'a> {
    fn len(&self) -> i32 {
        1
    }

    fn push_args(self, ctx: &Context) -> Result<()> {
        self.to_context(ctx)
    }
}

impl<'a> fmt::Display for Object<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.refer.ctx.get_global_string("JSON");
        let j: Object = Object::from_context(self.refer.ctx, -1).unwrap();
        self.refer.ctx.pop(1);
        let clone = self.clone();
        let json = j
            .call::<_, _, String>("stringify", (clone, (), "  "))
            .unwrap();
        write!(f, "{}", json)
    }
}
