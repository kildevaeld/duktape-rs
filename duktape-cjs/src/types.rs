use super::error::Result;
use duktape::Context;

pub trait ModuleLoader {
    fn resolve(&self, id: &str) -> Result<String>;
    fn read(&self, id: &str) -> Result<Vec<u8>>;
}

pub trait ModuleResolver {
    fn load(&self, ctx: &mut Context, buffer: &[u8]) -> Result<()>;
}

pub static KEY: &'static [u8] = b"commonjs";
