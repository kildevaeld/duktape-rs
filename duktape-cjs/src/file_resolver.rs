use super::error::{ErrorKind, Result};
use super::utils;
use super::ModuleResolver;
use std::fs;
use std::path::{self, PathBuf};
pub struct FileResolver;

macro_rules! to_string {
    ($id: ident) => {{
        $id.to_str().unwrap_or("").to_owned()
    }};
}

macro_rules! resolve_err {
    ($id: ident) => {
        return Err(ErrorKind::Resolve(to_string!($id)).into());
    };
}

impl ModuleResolver for FileResolver {
    fn read(&self, id: &str) -> Result<Vec<u8>> {
        Ok(fs::read(id)?)
    }

    fn resolve(&self, id: &str, parent: &str, extensions: &[String]) -> Result<String> {
        let parent = path::Path::new(parent);
        let mut id = PathBuf::from(id);

        if !id.is_absolute() {
            let parent_dir = parent.parent();
            if parent_dir.is_none() {
                resolve_err!(id);
            }

            id = utils::join(parent_dir.unwrap(), id.to_str().unwrap())?;
        }

        if !id.exists() && id.extension().is_none() {
            let mut found = false;
            for ext in extensions {
                let mut path = id.clone();
                path.set_extension(ext);
                if path.exists() {
                    id = path;
                    found = true;
                    break;
                }
            }
            if !found {
                resolve_err!(id);
            }
        } else if !id.exists() {
            resolve_err!(id);
        } else if id.is_dir() {
            let nid = id.join("index");
            let mut found = false;
            for ext in extensions {
                let mut path = nid.clone();
                path.set_extension(ext);
                if path.exists() {
                    id = path;
                    found = true;
                    break;
                }
            }
            if !found {
                resolve_err!(id);
            }
        }

        Ok(id.to_str().unwrap().to_owned())
    }
}
