use super::error::Result;
use super::ModuleResolver;

pub struct FileResolver;

impl ModuleResolver for FileResolver {
    fn read(&self, id: &str) -> Result<Vec<u8>> {}

    fn resolve(&self, id: &str, parent: &str) -> Result<String> {}
}
