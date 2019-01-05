extern crate duktape;
extern crate duktape_modules;
#[macro_use]
extern crate bitflags;
#[cfg(feature = "http")]
extern crate reqwest;

mod builder;
mod fs;
#[cfg(feature = "http")]
mod http;
mod io;
mod process;
mod sources;

use duktape::prelude::*;
use duktape_modules::require;

// concat!(env!("OUT_DIR"), "/utils.js")

// pub static UTILS: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/utils.js"));
// pub static POLFILLS: &'static [u8] = include_bytes!("polyfill.js");
// pub static RUNTIME: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/runtime.js"));

pub use self::builder::Modules;

#[cfg(feature = "http")]
fn init_http(builder: &mut duktape_modules::Builder, config: &builder::Modules) {
    if config.contains(Modules::Http) {
        builder.module("http", http::init_http);
    }
}

pub fn register(ctx: &Context, builder: &mut duktape_modules::Builder, config: builder::Modules) {
    // ctx.eval(POLFILLS).unwrap();
    // ctx.pop(1);

    process::init_process(ctx).unwrap();

    if config.contains(Modules::Io) {
        builder.module("io", |ctx: &Context| io::init_io(ctx));
    }

    if config.contains(Modules::Fs) {
        builder.module("fs", |ctx: &Context| fs::init_fs(ctx));
    }

    if config.contains(Modules::Utils) {
        builder.module("utils", |ctx: &Context| {
            let module: Object = ctx.get(-1)?;
            require::eval_module(ctx, sources::UTILS, &module).unwrap();
            Ok(1)
        });
    }

    #[cfg(feature = "http")]
    init_http(builder, &config);
}

pub fn init_runtime(ctx: &Context) {
    ctx.compile_string(sources::RUNTIME, Compile::EVAL).unwrap();

    ctx.call(0).unwrap().pop(1);

    //ctx.push_global_object().call(1).unwrap();
}
