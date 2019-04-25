use super::super::error::DukResult;
use super::traits::ModuleResolver;
use pathutils::set_extname;
use std::io::Read;
use vfs::{ReadPath, VMetadata, VPath, VFS};

pub struct FileResolver<T: VFS> {
    vfs: T,
    cwd: String,
}

macro_rules! to_string {
    ($id: ident) => {{
        $id.to_string().to_owned()
    }};
}

macro_rules! resolve_err {
    ($id: ident) => {
        return duk_error!(to_string!($id));
    };
}

impl<T: VFS> ModuleResolver for FileResolver<T>
where
    <T as VFS>::Path: ReadPath,
{
    fn read(&self, id: &str) -> DukResult<Vec<u8>> {
        let path = self.vfs.path(id);
        let mut file = path.open()?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        Ok(buffer)
    }

    fn resolve(&self, id: &str, parent: &str, extensions: &[String]) -> DukResult<String> {
        let mut id = if id.chars().nth(0).unwrap() != '/' {
            let parent = if parent.is_empty() {
                self.vfs.path(&self.cwd)
            } else {
                let parent = pathutils::parent_path(&parent).unwrap_or_else(|| parent.to_string());
                self.vfs.path(&parent)
            };
            parent.resolve(id)
        } else {
            self.vfs.path(id)
        };

        if !id.exists() && id.extension().is_none() {
            let mut found = false;
            for ext in extensions {
                let mut path = id.clone();
                path = self.vfs.path(&set_extname(path.to_string().as_ref(), ext));

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
        } else if id.metadata()?.is_dir() {
            let nid = id.resolve("index");
            let mut found = false;
            for ext in extensions {
                let mut path = nid.clone();
                path = self.vfs.path(&set_extname(path.to_string().as_ref(), ext));
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

        Ok(id.to_string().to_string())
    }
}

pub fn file_resolver<T: VFS + 'static, S: AsRef<str>>(
    vfs: T,
    work_path: S,
) -> Box<dyn ModuleResolver>
where
    <T as VFS>::Path: ReadPath,
{
    return Box::new(FileResolver {
        vfs,
        cwd: work_path.as_ref().to_string(),
    });
}
