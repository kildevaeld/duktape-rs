use super::commonjs::CommonJS;
use super::error;
use super::internal;
use duktape::prelude::*;
use std::env;
use std::fs;
use std::path::Path;

pub trait CJSContext {
    fn eval_main<'a, T: AsRef<Path>>(&'a self, path: T) -> error::Result<Object<'a>>;
    fn eval_main_script<'a, T: AsRef<Path>, S: AsRef<[u8]>>(
        &'a self,
        path: T,
        script: S,
    ) -> error::Result<Object<'a>>;

    fn require<'a, Str: AsRef<[u8]>>(&'a self, name: Str) -> error::Result<Object<'a>>;
}

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

    let module = internal::push_module_object(ctx, &real_p, true)?;

    let common = ctx.data()?.get::<CommonJS>().unwrap();

    let ext = real_p.extension().unwrap();

    let loader = match common.loaders.iter().find(|m| m.extension.as_str() == ext) {
        Some(loader) => loader,

        None => bail!(error::ErrorKind::Resolve(format!(
            "no loader for: {:?}",
            ext
        ))),
    };

    // if loader.is_none() {
    //     internal::eval_module(ctx, script.as_ref(), &mut module)?;
    // }

    loader.loader.load(&ctx, &module, script.as_ref())?;

    Ok(module)
}

pub fn require<'a, Str: AsRef<[u8]>>(ctx: &'a Context, name: Str) -> error::Result<Object<'a>> {
    Ok(ctx
        .get_global_string("require")
        .push_string(name)
        .call(1)?
        .getp()?)
}

impl CJSContext for Context {
    fn eval_main<'a, T: AsRef<Path>>(&'a self, path: T) -> error::Result<Object<'a>> {
        eval_main(self, path)
    }

    fn eval_main_script<'a, T: AsRef<Path>, S: AsRef<[u8]>>(
        &'a self,
        path: T,
        script: S,
    ) -> error::Result<Object<'a>> {
        eval_main_script(self, path, script)
    }

    fn require<'a, Str: AsRef<[u8]>>(&'a self, name: Str) -> error::Result<Object<'a>> {
        require(self, name)
    }
}
