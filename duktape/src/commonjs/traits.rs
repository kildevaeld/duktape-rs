use super::super::error::DukResult;
use super::super::types::object::Object;

pub trait ModuleResolver {
    fn resolve(&self, id: &str, parent: &str, extensions: &[String]) -> DukResult<String>;
    fn read(&self, id: &str) -> DukResult<Vec<u8>>;
}

pub trait ModuleLoader {
    fn load(&self, module: &Object, buffer: &[u8]) -> DukResult<()>;
}

pub static KEY: &'static [u8] = b"commonjs";
pub static MODULE_ID_KEY: &'static [u8] = b"\xFFmoduleId";
