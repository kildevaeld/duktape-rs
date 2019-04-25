use super::super::context::Context;
use super::super::error::DukResult;
use super::super::from_context::*;
use super::super::types::object::Object;
use super::require::CommonJS;
use super::utils;
use std::env;
use std::path::Path;

pub trait ContextCommonJS {
    fn eval_main<'a, T: AsRef<Path>>(&'a self, path: T) -> DukResult<Object<'a>>;
    fn eval_main_script<'a, T: AsRef<str>, S: AsRef<[u8]>>(
        &'a self,
        path: T,
        script: S,
    ) -> DukResult<Object<'a>>;

    fn require<'a, Str: AsRef<[u8]>>(&'a self, name: Str) -> DukResult<Object<'a>>;
}

impl ContextCommonJS for Context {
    fn eval_main<'a, T: AsRef<Path>>(&'a self, path: T) -> DukResult<Object<'a>> {
        let mut real_p = path.as_ref().to_path_buf();
        if !real_p.is_absolute() {
            real_p = env::current_dir()?.join(path);
        }
        let content = std::fs::read(&real_p)?;
        self.eval_main_script(real_p.to_string_lossy().as_ref(), content)
    }

    fn eval_main_script<'a, T: AsRef<str>, S: AsRef<[u8]>>(
        &'a self,
        path: T,
        script: S,
    ) -> DukResult<Object<'a>> {
        let real_p = Path::new(path.as_ref());

        let module = utils::push_module_object(self, real_p, true)?;

        let common = self.data().get::<CommonJS>().unwrap();

        let ext = real_p.extension().unwrap();

        let loader = match common.loaders.iter().find(|m| m.extension.as_str() == ext) {
            Some(loader) => loader,
            None => return duk_uri_error!(format!("no loader for: {:?}", ext)),
        };

        loader.loader.load(&module, script.as_ref())?;

        Ok(module)
    }

    fn require<'a, Str: AsRef<[u8]>>(&'a self, name: Str) -> DukResult<Object<'a>> {
        Ok(self
            .get_global_string("require")
            .push_string(name)
            .call(1)?
            .getp()?)
    }
}
