use super::error::Result;
use super::internal;
use super::types::ModuleLoader;
use duktape::{
    types::{Object, Reference},
    Context,
};
use std::str;
pub struct JavascriptLoader;

impl ModuleLoader for JavascriptLoader {
    fn load(&self, ctx: &Context, module: &Object, buffer: &[u8]) -> Result<()> {
        internal::eval_module(ctx, buffer, module)?;
        Ok(())
    }
}

pub struct JsonLoader;

impl ModuleLoader for JsonLoader {
    fn load(&self, ctx: &Context, _module: &Object, buffer: &[u8]) -> Result<()> {
        let o = ctx.get_global_string("JSON").getp::<Object>()?;
        let json = str::from_utf8(buffer)?;
        o.call::<_, _, Reference>("parse", json)?.push();
        Ok(())
    }
}

pub fn javascript() -> Box<dyn ModuleLoader> {
    Box::new(JavascriptLoader {})
}

pub fn json() -> Box<dyn ModuleLoader> {
    Box::new(JsonLoader {})
}
