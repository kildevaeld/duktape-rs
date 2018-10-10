extern crate duktape;
extern crate duktape_cjs;
mod fs;
mod io;
mod process;

use duktape::error::{ErrorKind, Result};
use duktape::prelude::*;
use duktape_cjs::require;

pub static UTILS: &'static [u8] = include_bytes!("utils.js");
pub static POLFILLS: &'static [u8] = include_bytes!("polyfill.js");
pub static RUNTIME: &'static [u8] = include_bytes!("runtime.js");

pub fn init(ctx: &Context, builder: &mut duktape_cjs::RequireBuilder) {
    ctx.eval(POLFILLS).unwrap();
    ctx.pop(1);

    process::init_process(ctx).unwrap();

    builder
        .module("io", |ctx: &Context| {
            return io::init_io(ctx);
        })
        .module("fs", |ctx: &Context| {
            return fs::init_fs(ctx);
        })
        .module("utils", |ctx: &Context| {
            let module: Object = ctx.get(-1)?; //require::push_module_object(ctx, "utils", false).unwrap();
            require::eval_module(ctx, UTILS, &module).unwrap();
            Ok(1)
        });
}

pub fn init_runtime(ctx: &Context) {
    ctx.compile_string(RUNTIME, DUK_COMPILE_EVAL).unwrap();

    ctx.call(0).unwrap();

    ctx.push_global_object().call(1).unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
