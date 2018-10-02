use super::context::{Context, Idx, Type};
use super::encoding::{Deserialize, Serialize};
use super::error::{ErrorKind, Result};
use super::internal::{make_ref, push_ref, unref};
use duktape_sys as duk;
use std::fmt;
use std::iter;

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

impl<T1: 'static + Serialize, T2: 'static + Serialize, T3: 'static + Serialize> ArgumentList
    for (T1, T2, T3)
where
    &T1: Serialize,
    &T2: Serialize,
    &T3: Serialize,
{
    fn len(&self) -> i32 {
        3
    }
    fn push(self, ctx: &Context) {
        ctx.push(self.0);
        ctx.push(self.1);
        ctx.push(self.2);
    }
}

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
        self.ctx.pop(1);
        ret
    }

    pub fn get<T: Deserialize<'a>>(&self) -> Result<T> {
        self.push();
        let ret = self.ctx.get::<T>(-1);
        self.ctx.pop(1);
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
        Ok(Reference::new(ctx, index))
    }
}

impl<'a> fmt::Display for Reference<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.get_type() {
            Type::String => self.get::<String>().unwrap(),
            Type::Number => format!("{}", self.get::<u32>().unwrap()),
            _ => format!(""),
        };
        write!(f, "{}", s)
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
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> Deserialize<'a> for Object<'a> {
    fn from_context(ctx: &'a Context, index: i32) -> Result<Self> {
        Ok(Reference::new(ctx, index).into_object())
    }
}

pub struct Array<'a> {
    refer: Reference<'a>,
}

impl<'a> Array<'a> {
    pub(crate) fn new(refer: Reference<'a>) -> Array<'a> {
        Array { refer }
    }

    pub fn push<V: Serialize>(&self, value: V) -> &Self {
        self.refer.push();
        self.refer.ctx.push(value);
        self.refer.ctx.put_prop_index(-2, self.len() as u32);
        self.refer.ctx.pop(1);

        self
    }

    pub fn get<V: Deserialize<'a>>(&self, idx: u32) -> Result<V> {
        self.refer.push();
        self.refer.ctx.get_prop_index(-1, idx);

        let ret = self.refer.ctx.get::<V>(-1)?;

        self.refer.ctx.pop(2);

        Ok(ret)
    }

    pub fn len(&self) -> usize {
        self.refer.push();
        let ret = self.refer.ctx.get_length(-1);
        self.refer.ctx.pop(1);
        ret
    }

    pub fn iter(&'a self) -> impl iter::Iterator<Item = Reference<'a>> {
        ArrayIterator::new(self)
    }
}

impl<'a> Serialize for Array<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> Deserialize<'a> for Array<'a> {
    fn from_context(ctx: &'a Context, index: i32) -> Result<Self> {
        Reference::new(ctx, index).into_array()
    }
}

struct ArrayIterator<'a> {
    array: &'a Array<'a>,
    index: u32,
}

impl<'a> ArrayIterator<'a> {
    pub fn new(array: &'a Array<'a>) -> ArrayIterator<'a> {
        ArrayIterator {
            array: array,
            index: 0,
        }
    }
}

impl<'a> iter::Iterator for ArrayIterator<'a> {
    type Item = Reference<'a>;

    fn next(&mut self) -> Option<Reference<'a>> {
        if self.index == self.array.len() as u32 {
            return None;
        }

        let r = match self.array.get::<Reference>(self.index) {
            Ok(m) => m,
            Err(_) => return None,
        };

        self.index += 1;

        Some(r)
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
