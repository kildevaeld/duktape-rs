extern crate duktape;
extern crate duktape_modules;

static SOURCE: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/buble.js"));
static RUNTIME: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/es6.shim.js"));

use duktape::prelude::*;
use duktape_modules::{require, CJSContext};
use std::str;

struct Es6Loader {}

impl duktape_modules::ModuleLoader for Es6Loader {
    fn load(
        &self,
        ctx: &Context,
        module: &Object,
        buffer: &[u8],
    ) -> duktape_modules::error::Result<()> {
        let buble = ctx.require("es2015")?;
        let source = str::from_utf8(buffer)?;

        // let options: Object = ctx.create()?;
        // options.set("presets", vec!["es2015"]);
        // options.set("plugins", vec!["transform-decorators-legacy"]);

        let options: Object = ctx.create()?;
        let transforms: Object = ctx.create()?;
        transforms.set("dangerousForOf", true);
        options.set("transforms", transforms);

        let out = buble.call::<_, _, Object>("transform", (source, options))?;
        let code = out.get::<_, &str>("code")?;

        require::eval_module(&ctx, code.as_bytes(), module)?;

        Ok(())
    }
}

pub fn register(ctx: &Context, builder: &mut duktape_modules::Builder) {
    ctx.compile_string(RUNTIME, Compile::EVAL).unwrap();
    ctx.call(0).unwrap().pop(1);

    builder.module("es2015", |ctx: &Context| {
        let module: Object = ctx.get(-1)?;
        require::eval_module(ctx, SOURCE, &module).unwrap();
        Ok(1)
    });

    builder.loader("es6", Box::new(Es6Loader {}));
    builder.loader("js", Box::new(Es6Loader {}));
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
