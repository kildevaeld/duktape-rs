use super::types::{ModuleLoader, ModuleResolver};
use super::types::{KEY, MODULE_ID_KEY};
// use duktape::{error::ErrorKind, error::Result, Callable, Context};
use super::internal;
use duktape::prelude::*;
use duktape::Key;
use regex::Regex;
use std::path::Path;

pub struct CommonJS {
    loaders: Vec<Loader>,
    resolvers: Vec<Resolver>,
    modules: Vec<Module>,
}

impl CommonJS {
    pub fn build_require<'a>(&self, ctx: &'a Context, module_id: &str) -> Result<Function<'a>> {
        let require: Box<dyn Callable> = Box::new(Require {});
        let function: Object = ctx.push(require).getp()?;

        let mut stash: Object = ctx.push_global_stash().getp()?;
        stash = stash.get(KEY)?;

        function
            .set(MODULE_ID_KEY, module_id)
            .set("cache", stash.get::<_, Reference>("cache")?)
            .set("main", stash.get::<_, Reference>("main")?);

        let function: Result<Function> = function.into();

        match function {
            Ok(mut ret) => {
                ret.set_name("require");
                Ok(ret)
            }
            Err(e) => Err(e),
        }
    }

    pub fn extensions(&self) -> Vec<String> {
        self.loaders
            .iter()
            .map(|m| m.extension.clone())
            .collect::<Vec<_>>()
    }
}

impl Drop for CommonJS {
    fn drop(&mut self) {
        println!("{}", "drop it like it's hard");
    }
}

impl Key for CommonJS {
    type Value = Self;
}

lazy_static! {
    static ref PROTOCOL_RE: Regex =
        Regex::new(r"^([a-zA-Z0-9]+)(?:://)(/?[a-zA-Z0-9\.\-]+(?:/[a-zA-Z0-9\.\-]+)*)$").unwrap();
    static ref FILE_RE: Regex = Regex::new(r"^(?:/|\.\.?/)(?:[^/\\0]+(?:/)?)+$").unwrap();
}

struct Loader {
    extension: String,
    loader: Box<dyn ModuleLoader>,
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
}

impl RequireBuilder {
    pub fn new() -> RequireBuilder {
        RequireBuilder {
            loaders: Vec::new(),
            resolvers: Vec::new(),
            modules: Vec::new(),
        }
    }

    pub fn loader(&mut self, extension: &str, loader: Box<dyn ModuleLoader>) -> &mut Self {
        self.loaders.push(Loader {
            extension: extension.to_owned(),
            loader: loader,
        });
        self
    }

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
    fn push_module(&self, ctx: &Context, id: &str) {
        ctx.push_object()
            .push(id)
            .put_prop_string(-2, "id")
            .push_object()
            .put_prop_string(-2, "exports");
    }

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
                .into())
            }
        };

        let o: Object = ctx.push_current_function().getp()?;
        let parent = o.get::<_, String>(MODULE_ID_KEY)?;

        let id = match resolver.resolver.resolve(idr, &parent, &repo.extensions()) {
            Ok(id) => id,
            Err(e) => return Err(ErrorKind::TypeError(format!("{}", e)).into()),
        };

        // if self.has_cache(ctx, &id) {
        //     return self.get_cache(ctx, &id);
        // }

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

        let found = found.unwrap();

        self.push_module(ctx, id);
        let module = internal::push_module_object(ctx, "", false).unwrap();

        let top = ctx.top();
        found.module.call(ctx)?;

        if ctx.top() > top {
            ctx.put_prop_string(-2, "exports");
            module.set("exports", ctx.get::<Reference>(-1)?);
        }

        Ok(module)
    }

    fn has_cache(&self, ctx: &Context, id: &str) -> bool {
        ctx.push_global_stash()
            .get_prop_string(-1, KEY)
            .get_prop_string(-1, "cache");
        let ret = ctx.has_prop_string(-1, id);
        ctx.pop(3);
        ret
    }

    // fn get_cache(&self, ctx: &Context, id: &str) -> Result<Object> {
    //     ctx.push_global_stash()
    //         .get_prop_string(-1, KEY)
    //         .get_prop_string(-1, "cache");

    //     if !ctx.has_prop_string(-1, id) {
    //         ctx.pop(3);
    //     } else {
    //         ctx.get_prop_string(-1, id);
    //     }

    //     ctx.getp()
    // }

    fn set_cache(&self, ctx: &Context, id: &str, index: i32) -> Result<()> {
        Ok(())
    }

    fn del_cache(&self, ctx: &Context, id: &str) -> Result<()> {
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

        // if ctx.top() == 0 {
        //     return Err(ErrorKind::TypeError("module return invalid type".to_string()).into());
        // }

        //id = ctx.get::<Object>(-1)?.get::<_, String>("id")?;

        //.set_cache(ctx, &id, -1)?;

        ctx.get_prop_string(-1, "exports");

        Ok(1)
    }
}

impl Drop for Require {
    fn drop(&mut self) {}
}

pub fn build_require<'a>(ctx: &'a Context, module_id: &str) -> Result<Function<'a>> {
    let require: Box<dyn Callable> = Box::new(Require {});
    let function: Object = ctx.push(require).getp()?;

    let mut stash: Object = ctx.push_global_stash().getp()?;
    stash = stash.get(KEY)?;

    function
        .set(MODULE_ID_KEY, module_id)
        .set("cache", stash.get::<_, Reference>("cache")?)
        .set("main", stash.get::<_, Reference>("main")?);

    let function: Result<Function> = function.into();

    match function {
        Ok(mut ret) => {
            ret.set_name("require");
            Ok(ret)
        }
        Err(e) => Err(e),
    }
}
