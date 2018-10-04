use super::context::{Context, Idx, Type};
use super::encoding::{Deserialize, Serialize};
use super::error::Result;
use super::privates::{make_ref, push_ref, unref};
use duktape_sys as duk;
use std::fmt;

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
