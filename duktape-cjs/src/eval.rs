use super::error;
use super::internal;
use duktape::prelude::*;
use std::fs;
use std::path::Path;

pub fn eval_main<'a, T: AsRef<Path>>(
    ctx: &'a mut duktape::Context,
    path: T,
) -> error::Result<Object> {
    let path = path.as_ref();
    let content = fs::read(path)?;
    eval_main_script(ctx, path, content)
}

pub fn eval_main_script<'a, T: AsRef<Path>, S: AsRef<[u8]>>(
    ctx: &'a mut duktape::Context,
    path: T,
    script: S,
) -> error::Result<Object> {
    let mut module = internal::push_module_object(ctx, path, true)?;
    internal::eval_module(ctx, script.as_ref(), &mut module)?;

    Ok(module)
}
