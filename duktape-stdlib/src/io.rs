use duktape;
use duktape::prelude::*;
use std::io::{self, Write};

pub fn init_io(ctx: &mut Context) -> Result<i32> {
    let mut module = ctx.create::<Object>()?;

    let mut stdout = duktape::class::Builder::default();
    stdout
        .method("write", |ctx, this| {
            if ctx.is(Type::Undefined, 0) {
                return Err(ErrorKind::TypeError("invalid type".to_owned()).into());
            }

            let r = ctx.get::<Reference>(0)?;
            write!(io::stdout(), "{}", r);

            ctx.push_this();
            Ok(1)
        })
        .method("flush", |ctx, this| {
            io::stdout().flush();
            ctx.push_this();
            Ok(1)
        });

    module.set("Stdout", stdout);

    let out: Object = module.construct("Stdout", ())?;

    module.set("stdout", out);

    ctx.push(module);

    Ok(1)
}
