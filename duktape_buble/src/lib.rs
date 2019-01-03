extern crate duktape;
extern crate duktape_cjs;

static SOURCE: &'static [u8] = include_bytes!("../buble/dist/buble.js");

use duktape::prelude::*;
use duktape_cjs::{require, CJSContext};
use std::str;

struct Es6Loader {}

impl duktape_cjs::ModuleLoader for Es6Loader {
    fn load(
        &self,
        ctx: &Context,
        module: &Object,
        buffer: &[u8],
    ) -> duktape_cjs::error::Result<()> {
        let buble = ctx.require("buble")?;
        let source = str::from_utf8(buffer)?;
        let out = buble.call::<_, _, Object>("transform", source)?;
        let code = out.get::<_, &str>("code")?;

        require::eval_module(&ctx, code.as_bytes(), module)?;

        Ok(())
    }
}

pub fn register(_ctx: &Context, builder: &mut duktape_cjs::Builder) {
    builder.module("buble", |ctx: &Context| {
        let module: Object = ctx.get(-1)?;
        require::eval_module(ctx, SOURCE, &module).unwrap();
        Ok(1)
    });

    builder.loader("es6", Box::new(Es6Loader {}));
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
