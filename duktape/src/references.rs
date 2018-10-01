use super::context::{Context, Idx, Type};
use super::encoding::{Deserialize, Serialize};
use super::error::{ErrorKind, Result};
use super::internal::{make_ref, push_ref, unref};
use duktape_sys as duk;
use std::ops::Index;

pub trait ArgumentList {
    fn len(&self) -> i32;
    fn push(self, ctx: &Context);
}

impl<'a> ArgumentList for &'a str {
    fn len(&self) -> i32 {
        1
    }

    fn push(self, context: &Context) {
        let len = self.len();
        let data = self.as_ptr() as *const i8;
        unsafe {
            duktape_sys::duk_push_lstring(context.inner, data, len);
        };
    }
}

impl<T1: 'static + Serialize, T2: 'static + Serialize> ArgumentList for (T1, T2)
where
    &T1: Serialize,
    &T2: Serialize,
{
    fn len(&self) -> i32 {
        2
    }
    fn push(self, ctx: &Context) {
        ctx.push(self.0);
        ctx.push(self.1);
    }
}

// impl ArgumentList for () {
//     fn len(&self) -> i32 {
//         0
//     }
//     fn push(self, ctx: &Context) {}
// }

pub struct Reference<'a> {
    pub(crate) ctx: &'a Context,
    refer: u32,
}

impl<'a> Reference<'a> {
    pub fn new(ctx: &'a Context, idx: Idx) -> Reference<'a> {
        unsafe { duk::duk_dup(ctx.inner, idx) };
        let refer = unsafe { make_ref(ctx.inner) };
        Reference { ctx, refer }
    }

    pub fn into_object(self) -> Object<'a> {
        Object::new(self)
    }

    pub fn into_array(self) -> Result<Array<'a>> {
        if !self.is(Type::Array) {
            return Err(ErrorKind::TypeError("not an array".to_string()).into());
        }
        Ok(Array::new(self))
    }

    pub fn into_function(self) -> Result<Function<'a>> {
        if !self.is(Type::Function) {
            return Err(ErrorKind::TypeError("not a function".to_string()).into());
        }
        Ok(Function::new(self))
    }

    pub fn get_type(&self) -> Type {
        unsafe { push_ref(self.ctx.inner, self.refer) };
        let ret = self.ctx.get_type(-1);
        unsafe { duk::duk_pop(self.ctx.inner) };
        ret
    }

    pub fn is(&self, t: Type) -> bool {
        self.get_type() == t
    }

    pub fn push(&self) -> &Self {
        unsafe { push_ref(self.ctx.inner, self.refer) };
        self
    }
}

impl<'a> Serialize for Reference<'a> {
    fn to_context(self, ctx: &Context) -> Result<()> {
        unsafe { push_ref(ctx.inner, self.refer) };
        Ok(())
    }
}

impl<'a> Deserialize<'a> for Reference<'a> {
    fn from_context(ctx: &'a Context, index: i32) -> Result<Self> {
        Ok(Reference::new(ctx, -1))
    }
}

impl<'a> Drop for Reference<'a> {
    fn drop(&mut self) {
        unsafe { unref(self.ctx.inner, self.refer) };
    }
}

pub struct Object<'a> {
    refer: Reference<'a>,
}

impl<'a> Object<'a> {
    pub(crate) fn new(refer: Reference<'a>) -> Object<'a> {
        Object { refer }
    }

    pub fn get<T: AsRef<[u8]>, V: Deserialize<'a>>(&self, prop: T) -> Result<V> {
        unsafe { push_ref(self.refer.ctx.inner, self.refer.refer) };
        let ret = self.refer.ctx.get_prop_string(-1, prop).get::<V>(-1)?;
        unsafe { duk::duk_pop_n(self.refer.ctx.inner, 2) };
        Ok(ret)
    }

    pub fn set<T: AsRef<[u8]>, V: Serialize>(&mut self, prop: T, value: V) -> &mut Self {
        unsafe { push_ref(self.refer.ctx.inner, self.refer.refer) };
        self.refer.ctx.push(value).put_prop_string(-2, prop).pop(1);
        self
    }

    pub fn del<T: AsRef<[u8]>>(&mut self, prop: T) -> &mut Self {
        unsafe { push_ref(self.refer.ctx.inner, self.refer.refer) };
        self.refer.ctx.del_prop_string(-1, prop.as_ref());
        unsafe {
            duk::duk_del_prop_string(
                self.refer.ctx.inner,
                -1,
                prop.as_ref().as_ptr() as *const i8,
            )
        };
        unsafe { duk::duk_pop(self.refer.ctx.inner) };
        self
    }

    pub fn as_ref(&self) -> &'a Reference {
        &self.refer
    }

    pub fn call<T: AsRef<str>, A: ArgumentList, R: Deserialize<'a>>(
        &self,
        fn_name: T,
        args: A,
    ) -> Result<R> {
        self.refer.push();
        let idx = self.refer.ctx.normalize_index(-1);
        self.refer.ctx.push(fn_name.as_ref());
        let len = args.len();
        args.push(self.refer.ctx);
        if let Err(e) = self.refer.ctx.call_prop(idx, len) {
            self.refer.ctx.pop(1);
            return Err(e);
        }
        self.refer.ctx.remove(-2);
        self.refer.ctx.getp()
    }
}

impl<'a> Serialize for Object<'a> {
    fn to_context(self, ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> Deserialize<'a> for Object<'a> {
    fn from_context(ctx: &'a Context, index: i32) -> Result<Self> {
        Ok(Reference::new(ctx, -1).into_object())
    }
}

pub struct Array<'a> {
    refer: Reference<'a>,
}

impl<'a> Array<'a> {
    pub(crate) fn new(refer: Reference<'a>) -> Array<'a> {
        Array { refer }
    }
}

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
