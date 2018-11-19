use super::super::context::{Context, Idx, Type};
use super::super::error::Result;
use super::super::privates::{make_ref, push_ref, unref};
use super::{FromDuktape, ToDuktape};
use duktape_sys as duk;
use std::ffi::CStr;
use std::fmt;

pub struct Ref<'a> {
    pub(crate) ctx: &'a Context,
    refer: u32,
}

impl<'a> Ref<'a> {
    pub(crate) fn new(ctx: &'a Context, idx: Idx) -> Ref<'a> {
        unsafe { duk::duk_dup(ctx.inner, idx) };
        let refer = unsafe { make_ref(ctx.inner) };
        Ref { ctx, refer }
    }

    pub fn get_type(&self) -> Type {
        unsafe { push_ref(self.ctx.inner, self.refer) };
        let ret = self.ctx.get_type(-1);
        self.ctx.pop(1);
        ret
    }

    pub fn is(&self, t: Type) -> bool {
        self.get_type() == t
    }

    pub fn get<T: FromDuktape<'a>>(&self) -> Result<T> {
        self.push();
        let ret = T::from_context(self.ctx, -1);
        self.ctx.pop(1);
        ret
    }

    pub fn push(&self) -> &Self {
        unsafe { push_ref(self.ctx.inner, self.refer) };
        self
    }

    pub fn instance_of(&self, reference: &Ref) -> bool {
        self.push();
        reference.push();
        let ret = self.ctx.instance_of(-2, -1);
        self.ctx.pop(2);
        ret
    }
}

impl<'a> ToDuktape for Ref<'a> {
    fn to_context(self, ctx: &Context) -> Result<()> {
        unsafe { push_ref(ctx.inner, self.refer) };
        Ok(())
    }
}

impl<'a> ToDuktape for &'a Ref<'a> {
    fn to_context(self, ctx: &Context) -> Result<()> {
        unsafe { push_ref(ctx.inner, self.refer) };
        Ok(())
    }
}

impl<'a> FromDuktape<'a> for Ref<'a> {
    fn from_context(ctx: &'a Context, index: i32) -> Result<Self> {
        Ok(Ref::new(ctx, index))
    }
}

impl<'a> fmt::Display for Ref<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // let s = match self.get_type() {
        //     Type::String => self.get::<String>().unwrap(),
        //     Type::Number => format!("{}", self.get::<u32>().unwrap()),
        //     _ => format!(""),
        // };
        self.push();
        let s = unsafe { CStr::from_ptr(duk::duk_safe_to_string(self.ctx.inner, -1)) };
        self.ctx.pop(1);
        write!(f, "{}", s.to_string_lossy())
    }
}

impl<'a> Drop for Ref<'a> {
    fn drop(&mut self) {
        unsafe { unref(self.ctx.inner, self.refer) };
    }
}

impl<'a> Clone for Ref<'a> {
    fn clone(&self) -> Self {
        self.push();
        let r = Ref::new(self.ctx, -1);
        self.ctx.pop(1);
        r
    }
}
