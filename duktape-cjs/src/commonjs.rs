use super::types::KEY;
use super::types::{ModuleLoader, ModuleResolver};
use duktape::{error::ErrorKind, error::Result, Callable, Context};
use regex::Regex;
use std::path::Path;

lazy_static! {
    static ref PROTOCOL_RE: Regex =
        Regex::new(r"^([a-zA-Z0-9]+)(?:://)(/?[a-zA-Z0-9\.\-]+(?:/[a-zA-Z0-9\.\-]+)*)$").unwrap();
    static ref FILE_RE: Regex = Regex::new(r"^(?:/|\.\.?/)(?:[^/\\0]+(?:/)?)+$").unwrap();
}

struct Loader {
    protocol: String,
    loader: Box<dyn ModuleLoader>,
}

struct Resolver {
    extention: String,
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

    pub fn loader(&mut self, loader: Box<dyn ModuleLoader>) -> &mut Self {
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

    pub fn build(self) -> Box<dyn Callable> {
        let req = Require {
            loaders: self.loaders,
            modules: self.modules,
            resolvers: self.resolvers,
        };

        Box::new(req)
    }
}

pub struct Require {
    loaders: Vec<Loader>,
    resolvers: Vec<Resolver>,
    modules: Vec<Module>,
}

impl Require {
    fn push_module(&self, ctx: &mut Context, id: &str) {
        ctx.push_object()
            .push(id)
            .put_prop_string(-2, "id")
            .push_object()
            .put_prop_string(-2, "exports");
    }

    fn load_module(&self, id: &str) -> Result<()> {
        let caps = PROTOCOL_RE.captures(id).unwrap();

        let protocol = caps.get(1).unwrap().as_str();
        let resolver = match self
            .loaders
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
        Ok(())
    }

    fn load_builtin_module(&self, id: &str, ctx: &mut Context) -> Result<()> {
        // Find buildin
        let found = (&self.modules).into_iter().find(|m| m.name == id);
        if found.is_none() {
            return Err(ErrorKind::TypeError(format!("could not find module: '{}'", id)).into());
        }

        let found = found.unwrap();

        self.push_module(ctx, id);

        let top = ctx.top();
        found.module.call(ctx)?;

        if ctx.top() > top {
            ctx.put_prop_string(-2, "exports");
        }

        Ok(())
    }

    fn has_cache(&self, ctx: &mut Context, id: &str) -> bool {
        ctx.push_global_stash()
            .get_prop_string(-1, KEY)
            .get_prop_string(-1, "cache");
        let ret = ctx.has_prop_string(-1, id);
        ctx.pop(3);
        ret
    }

    fn get_cache(&self, ctx: &mut Context, id: &str) -> Result<()> {
        Ok(())
    }

    fn set_cache(&self, ctx: &mut Context, id: &str, index: i32) -> Result<()> {
        Ok(())
    }

    fn del_cache(&self, ctx: &mut Context, id: &str) -> Result<()> {
        Ok(())
    }
}

impl Callable for Require {
    fn argc(&self) -> i32 {
        1
    }
    fn call(&self, ctx: &mut Context) -> Result<i32> {
        if !ctx.is_string(0) {
            return Err(ErrorKind::TypeError("string expected".to_string()).into());
        }

        let mut id: String = ctx.get(0)?;
        ctx.pop(1);

        if (&self.modules).into_iter().find(|m| m.name == id).is_some() {
            if self.has_cache(ctx, &id) {
                self.get_cache(ctx, &id)?;
            } else {
                self.load_builtin_module(&id, ctx)?;
            }
        } else {
            if FILE_RE.is_match(&id) {
                id = format!("file://{}", id);
            }

            if !PROTOCOL_RE.is_match(&id) {
                return Err(ErrorKind::TypeError(format!("invalid require id: {}", id)).into());
            }
            if self.has_cache(ctx, &id) {
                self.get_cache(ctx, &id);
            } else {
                self.load_module(&id)?;
            }
        }

        if ctx.top() == 0 {
            return Err(ErrorKind::TypeError("module return invalid type".to_string()).into());
        }

        ctx.get_prop_string(-1, "exports");

        Ok(1)
    }
}

impl Drop for Require {
    fn drop(&mut self) {}
}
