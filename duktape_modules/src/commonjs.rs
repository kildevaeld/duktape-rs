use super::types::{ModuleLoader, ModuleResolver};
use super::types::{KEY, MODULE_ID_KEY};
// use duktape::{error::ErrorKind, error::Result, Callable, Context};
use super::internal;
use duktape::prelude::*;
use duktape::Key;
use duktape::{error::ErrorKind, error::Result};
use regex::Regex;
use std::path::Path;

pub struct CommonJS {
    pub(crate) loaders: Vec<Loader>,
    resolvers: Vec<Resolver>,
    modules: Vec<Module>,
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
}

impl Drop for CommonJS {
    fn drop(&mut self) {}
}

impl Key for CommonJS {
    type Value = Self;
}

lazy_static! {
    static ref PROTOCOL_RE: Regex =
        Regex::new(r"^([a-zA-Z0-9]+)(?:://)(/?[a-zA-Z0-9\.\-]+(?:/[a-zA-Z0-9\.\-]+)*)$").unwrap();
    static ref FILE_RE: Regex = Regex::new(r"^(?:/|\.\.?/)(?:[^/\\0]+(?:/)?)+$").unwrap();
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

pub struct Builder {
    loaders: Vec<Loader>,
    resolvers: Vec<Resolver>,
    modules: Vec<Module>,
    pub(crate) file_loader: bool,
}

impl Builder {
    /// Creates a new builder
    pub fn new() -> Builder {
        Builder {
            loaders: Vec::new(),
            resolvers: Vec::new(),
            modules: Vec::new(),
            file_loader: true,
        }
    }

    // Add a loader to the builder
    pub fn loader(&mut self, extension: &str, loader: Box<dyn ModuleLoader>) -> &mut Self {
        self.loaders.push(Loader {
            extension: extension.to_owned(),
            loader: loader,
        });
        self
    }

    // Add a resovler to the builder
    pub fn resolver<T: AsRef<str>>(
        &mut self,
        protocol: T,
        resolver: Box<dyn ModuleResolver>,
    ) -> &mut Self {
        self.resolvers.push(Resolver {
            protocol: protocol.as_ref().to_owned(),
            resolver: resolver,
        });
        self
    }

    // Add a builtin module
    pub fn module<T: 'static>(&mut self, id: &str, module: T) -> &mut Self
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

    pub fn file_loader(&mut self, enable: bool) -> &mut Self {
        self.file_loader = enable;
        self
    }

    // Build
    pub fn build(self) -> CommonJS {
        CommonJS {
            loaders: self.loaders,
            modules: self.modules,
            resolvers: self.resolvers,
        }
    }
}

pub struct Require;

impl Require {
    /// Load a module
    fn load_module<'a>(&self, id: &str, ctx: &'a Context, repo: &CommonJS) -> Result<Object<'a>> {
        let caps = PROTOCOL_RE.captures(id).unwrap();

        let protocol = caps.get(1).unwrap().as_str();
        let idr = caps.get(2).unwrap().as_str();
        let resolver = match repo
            .resolvers
            .iter()
            .find(|m| m.protocol.as_str() == protocol)
        {
            Some(resolver) => resolver,
            None => {
                return Err(ErrorKind::TypeError(format!(
                    "could not find resolver for protocol: '{}'",
                    protocol
                ))
                .into());
            }
        };

        let o: Object = ctx.push_current_function().getp()?;
        let parent = o.get::<_, String>(MODULE_ID_KEY)?;

        let id = match resolver.resolver.resolve(idr, &parent, &repo.extensions()) {
            Ok(id) => id,
            Err(e) => return Err(ErrorKind::TypeError(format!("{}", e)).into()),
        };

        if self.has_cache(ctx, &id)? {
            return self.get_cache(ctx, &id);
        }

        let path = Path::new(&id);

        let module = match internal::push_module_object(ctx, &path, false) {
            Ok(id) => id,
            Err(e) => return Err(ErrorKind::Error(format!("{}", e)).into()),
        };

        if path.extension().is_none() {
            bail!(ErrorKind::TypeError(format!(
                "could not infer extension for path {}",
                id
            )));
        }

        let ext = path.extension().unwrap();

        let loader = match repo.loaders.iter().find(|m| m.extension.as_str() == ext) {
            Some(loader) => loader,
            None => bail!(ErrorKind::Error(format!("no loader for: {:?}", ext))),
        };

        let content = match resolver.resolver.read(&id) {
            Err(e) => bail!(ErrorKind::Error(format!("{}", e))),
            Ok(m) => m,
        };

        match loader.loader.load(ctx, &module, &content) {
            Err(e) => bail!(ErrorKind::Error(format!("{}", e))),
            Ok(_) => Ok(module),
        }
    }

    /// Load a builtin module
    fn load_builtin_module<'a>(
        &self,
        id: &str,
        ctx: &'a Context,
        repo: &CommonJS,
    ) -> Result<Object<'a>> {
        // Find buildin
        let found = repo.modules.iter().find(|m| m.name == id);
        if found.is_none() {
            return Err(ErrorKind::TypeError(format!("could not find module: '{}'", id)).into());
        }

        if self.has_cache(ctx, id)? {
            return self.get_cache(ctx, id);
        }

        let found = found.unwrap();

        let module = internal::push_module_object(ctx, id, false).unwrap();
        module.clone().to_context(ctx)?;
        let top = ctx.top();

        found.module.call(ctx)?;

        if ctx.top() > top {
            module.set("exports", ctx.getp::<Ref>()?);
        }
        ctx.pop(1);
        Ok(module)
    }

    fn has_cache(&self, ctx: &Context, id: &str) -> Result<bool> {
        let cache = ctx
            .push_global_stash()
            .getp::<Object>()?
            .get::<_, Object>(KEY)?
            .get::<_, Object>("cache")?;

        Ok(cache.has(id))
    }

    fn get_cache<'a>(&self, ctx: &'a Context, id: &str) -> Result<Object<'a>> {
        let cache = ctx
            .push_global_stash()
            .getp::<Object>()?
            .get::<_, Object>(KEY)?
            .get::<_, Object>("cache")?;

        Ok(cache.get::<_, Object>(id).unwrap())
    }

    fn set_cache(&self, ctx: &Context, id: &str, module: &Object) -> Result<()> {
        let cache = ctx
            .push_global_stash()
            .getp::<Object>()?
            .get::<_, Object>(KEY)?
            .get::<_, Object>("cache")?;

        cache.set(id, module);

        Ok(())
    }
}

impl Callable for Require {
    fn argc(&self) -> i32 {
        1
    }
    fn call(&self, ctx: &Context) -> Result<i32> {
        if !ctx.is_string(0) {
            return Err(ErrorKind::TypeError("string expected".to_string()).into());
        }

        let mut id: String = ctx.get(0)?;
        ctx.pop(1);

        let common = ctx.data()?.get::<CommonJS>().unwrap();

        let module = if common.modules.iter().find(|m| m.name == id).is_some() {
            self.load_builtin_module(&id, ctx, common)?
        } else {
            if FILE_RE.is_match(&id) {
                id = format!("file://{}", id);
            }
            if !PROTOCOL_RE.is_match(&id) {
                return Err(ErrorKind::TypeError(format!("invalid require id: {}", id)).into());
            }
            self.load_module(&id, ctx, &common)?
        };

        if !module.has("exports") {
            bail!(ErrorKind::TypeError(format!(
                "module does not have a 'exports' field"
            )));
        }

        self.set_cache(ctx, &id, &module)?;

        module.get::<_, Ref>("exports")?.push();

        Ok(1)
    }
}

impl Drop for Require {
    fn drop(&mut self) {}
}

pub(crate) fn build_require<'a>(ctx: &'a Context, module_id: &str) -> Result<Function<'a>> {
    let function: Object = ctx.push_function(Require {}).getp()?;

    let mut stash: Object = ctx.push_global_stash().getp()?;
    stash = stash.get(KEY)?;

    function
        .set(MODULE_ID_KEY, module_id)
        .set("cache", stash.get::<_, Ref>("cache")?)
        .set("main", stash.get::<_, Ref>("main")?);

    let function: Result<Function> = function.into();

    match function {
        Ok(mut ret) => {
            ret.set_name("require");
            Ok(ret)
        }
        Err(e) => Err(e),
    }
}
