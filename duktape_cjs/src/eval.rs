use super::error;
use super::internal;
use duktape::prelude::*;
use std::env;
use std::fs;
use std::path::Path;

pub fn eval_main<'a, T: AsRef<Path>>(ctx: &'a duktape::Context, path: T) -> error::Result<Object> {
    let mut real_p = path.as_ref().to_path_buf();
    if !real_p.is_absolute() {
        real_p = env::current_dir()?.join(path);
    }
    let content = fs::read(&real_p)?;
    eval_main_script(ctx, real_p, content)
}

pub fn eval_main_script<'a, T: AsRef<Path>, S: AsRef<[u8]>>(
    ctx: &'a duktape::Context,
    path: T,
    script: S,
) -> error::Result<Object> {
    let mut real_p = path.as_ref().to_path_buf();
    if !real_p.is_absolute() {
        real_p = env::current_dir()?.join(path);
    }
    let mut module = internal::push_module_object(ctx, real_p, true)?;
    internal::eval_module(ctx, script.as_ref(), &mut module)?;

    Ok(module)
}
