use super::super::callable::Callable;
use super::super::callable::*;
use super::super::context::{Context, Type};
use super::super::error::{DukError, DukResult};
use super::super::from_context::FromDuktapeContext;
use super::super::object::*;
use super::super::reference::{JSValue, Reference};
use super::super::to_context::*;
use super::env::Environment;
use super::loaders::{javascript, json};
use super::traits::*;
use super::utils;
use std::path::Path;
use typemap::Key;

pub struct CommonJS {
    pub(crate) loaders: Vec<Loader>,
    resolvers: Vec<Resolver>,
    modules: Vec<Module>,
    env: Option<Environment>,
}

impl CommonJS {
    pub fn extensions(&self) -> Vec<String> {
        self.loaders
            .iter()
            .map(|m| m.extension.clone())
            .collect::<Vec<_>>()
    }

    pub fn protocols(&self) -> Vec<String> {
        self.resolvers
            .iter()
            .map(|m| m.protocol.clone())
            .collect::<Vec<_>>()
    }

    pub fn modules(&self) -> Vec<String> {
        self.modules
            .iter()
            .map(|m| m.name.clone())
            .collect::<Vec<_>>()
    }

    pub fn env(&self) -> Option<&Environment> {
        match &self.env {
            Some(s) => Some(s),
            None => None,
        }
    }
}

impl Drop for CommonJS {
    fn drop(&mut self) {}
}

impl Key for CommonJS {
    type Value = Self;
}

pub(crate) struct Loader {
    pub(crate) extension: String,
    pub(crate) loader: Box<dyn ModuleLoader>,
}

struct Resolver {
    protocol: String,
    resolver: Box<dyn ModuleResolver>,
}

struct Module {
    name: String,
    module: Box<dyn Callable>,
}

pub struct RequireBuilder {
    loaders: Vec<Loader>,
    resolvers: Vec<Resolver>,
    modules: Vec<Module>,
    env: Option<Environment>,
}

impl RequireBuilder {
    // Add a loader to the builder
    pub fn loader(mut self, extension: &str, loader: Box<dyn ModuleLoader>) -> Self {
        self.loaders.push(Loader {
            extension: extension.to_owned(),
            loader: loader,
        });
        self
    }

    pub fn env(mut self, env: Environment) -> Self {
        self.env = Some(env);
        self
    }

    // Add a resovler to the builder
    pub fn resolver<T: AsRef<str>>(
        mut self,
        protocol: T,
        resolver: Box<dyn ModuleResolver>,
    ) -> Self {
        self.resolvers.push(Resolver {
            protocol: protocol.as_ref().to_owned(),
            resolver: resolver,
        });
        self
    }

    // Add a builtin module
    pub fn module<T: 'static>(mut self, id: &str, module: T) -> Self
    where
        T: Callable,
    {
        if self
            .modules
            .iter()
            .find(|m| m.name.as_str() == id)
            .is_some()
        {
            return self;
        }

        let boxed = Box::new(module);

        self.modules.push(Module {
            name: id.to_string(),
            module: boxed,
        });

        self
    }

    pub fn build_environment(&self, ctx: &Context) -> DukResult<()> {
        let env = self.env.as_ref().unwrap();
        let process = ctx.create::<Object>()?;
        let empty = Vec::<String>::new();

        process.set("cwd", env.cwd().to_str().unwrap_or(""))?.set(
            "args",
            match env.args() {
                Some(s) => s,
                None => &empty,
            },
        )?;

        let stdout = ctx.create::<Object>()?;

        stdout.set(
            "write",
            jsfunc((1, |ctx: &Context| {
                let cjs = ctx.data_mut().get_mut::<CommonJS>().unwrap();
                let pipes = match &mut cjs.env {
                    Some(s) => s.pipes_mut(),
                    None => return Ok(0),
                };

                let arg = match ctx.get_type(0) {
                    Type::String => ctx.get_string(0)?.as_bytes(),
                    Type::Buffer => ctx.get_bytes(0)?,
                    _ => return duk_type_error!(format!("invalid type {:?}", ctx.get_type(0))),
                };

                let size = pipes.stdout_mut().write(arg)?;
                ctx.push_uint(size as u32);

                Ok(1)
            })),
        )?;

        process.set("stdout", stdout)?;

        ctx.push_global_object();
        process.push();
        ctx.put_prop_string(-2, "process");

        Ok(())
    }

    // Build
    pub fn build(self, ctx: &Context) -> DukResult<bool> {
        ctx.push_global_stash();

        if ctx.has_prop_string(-1, KEY) {
            return Ok(false);
        }

        ctx.push_bare_object()
            .push_bare_object()
            .put_prop_string(-2, "cache")
            .put_prop_string(-2, KEY);

        ctx.pop(1);

        if self.env.is_some() {
            self.build_environment(ctx)?;
        }

        let cjs = CommonJS {
            loaders: self.loaders,
            modules: self.modules,
            resolvers: self.resolvers,
            env: self.env,
        };

        ctx.data_mut().insert::<CommonJS>(cjs);

        ctx.push_global_object()
            .push(utils::build_require(ctx, "")?)?
            .put_prop_string(-2, "require")
            .pop(1);

        Ok(true)
    }
}

impl Default for RequireBuilder {
    fn default() -> Self {
        return RequireBuilder {
            loaders: vec![
                Loader {
                    loader: javascript(),
                    extension: "js".to_string(),
                },
                Loader {
                    loader: json(),
                    extension: "json".to_string(),
                },
            ],
            resolvers: Vec::new(),
            modules: Vec::new(),
            env: None,
        };
    }
}

pub struct Require;

impl Require {
    pub fn new() -> RequireBuilder {
        RequireBuilder::default()
    }

    /// Load a builtin module
    fn load_builtin_module<'a>(
        &self,
        id: &str,
        ctx: &'a Context,
        repo: &CommonJS,
    ) -> DukResult<(Object<'a>, String)> {
        // Find buildin
        let found = repo.modules.iter().find(|m| m.name == id);
        if found.is_none() {
            return duk_type_error!(format!("could not find module: '{}'", id));
        }

        if self.has_cache(ctx, id)? {
            let module = self.get_cache(ctx, id)?;
            return Ok((module, id.to_string()));
        }

        let found = found.unwrap();

        let module = utils::push_module_object(ctx, id, false).unwrap();
        module.clone().to_context(ctx)?;
        let top = ctx.top();

        found.module.call(ctx)?;

        if ctx.top() > top {
            module.set("exports", ctx.getp::<Reference>()?)?;
        }
        ctx.pop(1);
        Ok((module, id.to_string()))
    }

    fn load_module<'a>(
        &self,
        id: &str,
        protocol: &str,
        ctx: &'a Context,
        repo: &CommonJS,
    ) -> DukResult<(Object<'a>, String)> {
        let resolver = match repo
            .resolvers
            .iter()
            .find(|m| m.protocol.as_str() == protocol)
        {
            Some(resolver) => resolver,
            None => {
                return duk_type_error!("could not find resolver for protocol: '{}'");
            }
        };

        let o: Object = ctx.push_current_function().getp()?;
        let parent = o.get::<_, String>(MODULE_ID_KEY)?;

        let id = match resolver.resolver.resolve(id, &parent, &repo.extensions()) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };

        if self.has_cache(ctx, &id)? {
            let module = self.get_cache(ctx, &id)?;
            return Ok((module, id));
        }

        let path = Path::new(&id);

        let module = match utils::push_module_object(ctx, &path, false) {
            Ok(id) => id,
            Err(e) => return Err(DukError::with(e)),
        };

        if path.extension().is_none() {
            return duk_type_error!(format!("could not infer extension for path {}", id));
        }

        let ext = path.extension().unwrap();

        let loader = match repo.loaders.iter().find(|m| m.extension.as_str() == ext) {
            Some(loader) => loader,
            None => return duk_type_error!(format!("no loader for: {:?}", ext)),
        };

        let content = match resolver.resolver.read(&id) {
            Err(e) => return Err(DukError::with(e)),
            Ok(m) => m,
        };

        match loader.loader.load(&module, &content) {
            Err(e) => return Err(DukError::with(e)),
            Ok(_) => Ok((module, id)),
        }
    }

    fn has_cache(&self, ctx: &Context, id: &str) -> DukResult<bool> {
        let cache = ctx
            .push_global_stash()
            .getp::<Object>()?
            .get::<_, Object>(KEY)?
            .get::<_, Object>("cache")?;

        Ok(cache.has(id))
    }

    fn get_cache<'a>(&self, ctx: &'a Context, id: &str) -> DukResult<Object<'a>> {
        let cache = ctx
            .push_global_stash()
            .getp::<Object>()?
            .get::<_, Object>(KEY)?
            .get::<_, Object>("cache")?;

        Ok(cache.get::<_, Object>(id).unwrap())
    }

    fn set_cache(&self, ctx: &Context, id: &str, module: &Object) -> DukResult<()> {
        let cache = ctx
            .push_global_stash()
            .getp::<Object>()?
            .get::<_, Object>(KEY)?
            .get::<_, Object>("cache")?;

        cache.set(id, module)?;

        Ok(())
    }
}

impl Callable for Require {
    fn argc(&self) -> i32 {
        1
    }

    fn call(&self, ctx: &Context) -> DukResult<i32> {
        let opts: Object = ctx.get(0)?;

        let common = ctx.data().get::<CommonJS>().unwrap();

        let (module, id) = if opts.get::<_, Reference>("protocol")?.is(Type::Null) {
            self.load_builtin_module(opts.get("id")?, ctx, &common)
        } else {
            self.load_module(opts.get("id")?, opts.get("protocol")?, ctx, &common)
        }?;

        if !module.has("exports") {
            return duk_type_error!(format!("module does not have a 'exports' field"));
        }

        self.set_cache(ctx, &id, &module)?;

        module.get::<_, Reference>("exports")?.push();

        Ok(1)
    }
}

#[cfg(test)]
mod tests {

    use super::super::super::prelude::*;

    #[test]
    fn require() {
        let ctx = Context::new().unwrap();

        Require::new().build(&ctx).unwrap();
        assert!(ctx.eval("require('module')").is_err());
    }

    #[test]
    fn require_builtin() {
        let ctx = Context::new().unwrap();

        Require::new()
            .module(
                "module",
                (1, |ctx: &Context| {
                    ctx.push("Hello, World")?;
                    Ok(1)
                }),
            )
            .build(&ctx)
            .unwrap();

        assert!(ctx.eval("require('module')").is_ok());
        assert!(ctx.is_string(-1));
        assert_eq!(ctx.get_string(-1).unwrap(), "Hello, World");
    }
}
