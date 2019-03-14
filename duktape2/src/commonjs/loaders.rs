use super::super::context::Context;
use super::super::error::DukResult;
use super::super::from_context::FromDuktapeContext;
use super::super::object::{JSObject, Object};
use super::super::reference::{JSValue, Reference};
use super::traits::ModuleLoader;
use super::utils;
use std::str;

pub struct JavascriptLoader;

impl ModuleLoader for JavascriptLoader {
    fn load(&self, module: &Object, buffer: &[u8]) -> DukResult<()> {
        utils::eval_module(module.ctx(), buffer, module)?;
        Ok(())
    }
}

pub struct JsonLoader;

impl ModuleLoader for JsonLoader {
    fn load(&self, module: &Object, buffer: &[u8]) -> DukResult<()> {
        let o = module.ctx().get_global_string("JSON").getp::<Object>()?;
        let json = str::from_utf8(buffer)?;

        let r = o.prop("parse").call::<_, Reference>(json)?;
        module.set("exports", r)?;

        Ok(())
    }
}

pub fn javascript() -> Box<dyn ModuleLoader> {
    Box::new(JavascriptLoader {})
}

pub fn json() -> Box<dyn ModuleLoader> {
    Box::new(JsonLoader {})
}
