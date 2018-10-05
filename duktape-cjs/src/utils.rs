use super::error::{ErrorKind, Result};
use std::path::{Path, PathBuf};

pub fn join<T: AsRef<Path>, S: AsRef<str>>(base: T, cmp: S) -> Result<PathBuf> {
    let mut path: String = cmp.as_ref().to_owned();
    let mut base: PathBuf = base.as_ref().to_path_buf();
    if path.starts_with("./") {
        path = path.trim_left_matches("./").to_string();
    }

    while path.starts_with("../") {
        base = match base.parent() {
            Some(parent) => parent.to_path_buf(),
            None => return Err(ErrorKind::Resolve("".to_string()).into()),
        };
        path = path.trim_left_matches("..").to_string();
    }

    Ok(base.join(path))
}
