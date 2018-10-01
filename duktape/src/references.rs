use super::context::{Context, Idx, Type};
use super::encoding::{Deserialize, Serialize};
use super::error::{ErrorKind, Result};
use super::internal::{make_ref, push_ref, unref};
use duktape_sys as duk;
use std::ops::Index;

pub trait ArgumentList {
    fn len(&self) -> i32;
    fn push(&self, ctx: &mut Context);
}

impl<T1: Serialize> ArgumentList for (T1) {
    fn len(&self) -> i32 {
        1
    }
    fn push(&self, ctx: &mut Context) {}
}

impl ArgumentList for () {
    fn len(&self) -> i32 {
        0
    }
    fn push(&self, ctx: &mut Context) {}
}

pub struct Reference<'a> {
    pub(crate) ctx: &'a mut Context,
    refer: u32,
}

impl<'a> Reference<'a> {
    pub fn new(ctx: &'a mut Context, idx: Idx) -> Reference<'a> {
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
    fn to_context(self, ctx: &mut Context) -> Result<()> {
        unsafe { push_ref(ctx.inner, self.refer) };
        Ok(())
    }
}

// impl<'a> Clone for Reference<'a> {
//     fn clone(&self) -> Reference<'a> {
//         unsafe { push_ref(self.ctx.inner, self.refer) };
//         let refer = unsafe { make_ref(self.ctx.inner) };

//         unsafe { duk::duk_pop(self.ctx.inner) };
//         Reference {
//             ctx: self.ctx,
//             refer,
//         }
//     }
// }

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

    pub fn get<T: AsRef<[u8]>, V: Deserialize>(&self, prop: T) -> Result<V> {
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

    pub fn call<T: AsRef<[u8]>, A: ArgumentList>(&self, fn_name: T, args: A) -> Result<()> {
        Ok(())
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

    pub fn call<Args: ArgumentList, T: Deserialize>(&mut self, args: Args) -> Result<T> {
        self.refer.push();
        args.push(self.refer.ctx);
        self.refer.ctx.call(args.len())?;
        let ret = self.refer.ctx.getp()?;
        Ok(ret)
    }
}
