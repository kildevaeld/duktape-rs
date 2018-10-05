use super::error::Result;
use duktape::{types::Object, Context};

pub trait ModuleResolver {
    fn resolve(&self, id: &str, parent: &str, extensions: &[String]) -> Result<String>;
    fn read(&self, id: &str) -> Result<Vec<u8>>;
}

pub trait ModuleLoader {
    fn load(&self, ctx: &Context, module: &Object, buffer: &[u8]) -> Result<()>;
}

pub static KEY: &'static [u8] = b"commonjs";
pub static MODULE_ID_KEY: &'static [u8] = b"\xFFmoduleId";
