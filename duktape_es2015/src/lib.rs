static SOURCE: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/buble.js"));
static RUNTIME: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/es6.shim.js"));

use duktape::commonjs::utils::eval_module;
use duktape::prelude::*;
use std::str;

struct Es6Loader {}

impl ModuleLoader for Es6Loader {
    fn load(&self, module: &Object, buffer: &[u8]) -> DukResult<()> {
        let ctx = module.ctx();
        let buble = ctx.require("es2015")?;
        let source = str::from_utf8(buffer)?;

        // let options: Object = ctx.create()?;
        // options.set("presets", vec!["es2015"]);
        // options.set("plugins", vec!["transform-decorators-legacy"]);

        let options: Object = ctx.create()?;
        let transforms: Object = ctx.create()?;
        transforms.set("dangerousForOf", true)?;
        options.set("transforms", transforms)?;

        let out = buble
            .prop("transform")
            .call::<_, Object>((source, options))?;
        let code = out.get::<_, &str>("code")?;

        eval_module(&ctx, code.as_bytes(), module)?;

        Ok(())
    }
}

pub fn register(ctx: &Context, builder: RequireBuilder) -> DukResult<RequireBuilder> {
    ctx.compile_string(RUNTIME, Compile::EVAL)?;
    ctx.call(0)?.pop(1);

    Ok(builder
        .module("es2015", |ctx: &Context| {
            let module: Object = ctx.get(-1)?;
            eval_module(ctx, SOURCE, &module)?;
            Ok(1)
        })
        .loader("es6", Box::new(Es6Loader {}))
        .loader("js", Box::new(Es6Loader {})))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
