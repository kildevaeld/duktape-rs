use super::error::Result;
use super::ModuleResolver;

pub struct FileResolver;

impl ModuleResolver for FileResolver {
    fn read(&self, id: &str) -> Result<Vec<u8>> {
        Ok(vec![])
    }
    fn resolve(&self, id: &str, parent: &str) -> Result<String> {
        println!("parent: '{}', id: '{}'", parent, id);
        Ok(id.to_owned())
    }
}
