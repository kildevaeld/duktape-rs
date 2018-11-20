#[macro_use]
extern crate error_chain;
extern crate duktape;
#[macro_use]
extern crate lazy_static;
extern crate regex;

mod commonjs;
pub mod error;
mod eval;
mod file_resolver;
mod internal;
pub mod loaders;
mod types;
pub mod utils;

pub use self::commonjs::{Builder, CommonJS};
pub use self::eval::*;
pub use self::types::{ModuleLoader, ModuleResolver};

pub mod require {
    pub use super::internal::*;
}

pub mod resolvers {
    pub use super::file_resolver::*;
}

pub fn register(ctx: &duktape::Context, mut builder: Builder) -> duktape::error::Result<bool> {
    ctx.push_global_stash();
    if ctx.has_prop_string(-1, types::KEY) {
        return Ok(false);
    }

    ctx.push_bare_object()
        .push_bare_object()
        .put_prop_string(-2, "cache")
        .put_prop_string(-2, types::KEY);

    ctx.pop(1);

    if builder.file_loader {
        builder.resolver(
            "file",
            Box::new(file_resolver::FileResolver {}) as Box<dyn ModuleResolver>,
        );
    }

    builder
        .loader("js", loaders::javascript())
        .loader("json", loaders::json());

    ctx.data_mut()?.insert::<CommonJS>(builder.build());

    ctx.push_global_object()
        .push(commonjs::build_require(ctx, "")?)?
        .put_prop_string(-2, "require")
        .pop(1);

    Ok(true)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
