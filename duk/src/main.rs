extern crate duktape;
extern crate duktape_cjs;
extern crate duktape_stdlib;
extern crate env_logger;
extern crate log;
extern crate rustyline;

mod repl;

use duktape::prelude::*;
use std::{env, fs};

fn main() -> duktape_cjs::error::Result<()> {
    env_logger::init();

    let mut ctx = Context::new().unwrap();

    let mut require = duktape_cjs::RequireBuilder::new();

    duktape_stdlib::init(&ctx, &mut require, duktape_stdlib::Modules::default());

    duktape_cjs::register(&ctx, require)?;

    duktape_stdlib::init_runtime(&ctx);

    let args = env::args();

    if args.len() < 2 {
        return repl::run(&ctx);
    }

    let path = &args.collect::<Vec<String>>()[1];
    let data = fs::read(path).unwrap();
    ctx.push_global_stash();

    ctx.pop(1);
    duktape_cjs::eval_main_script(&mut ctx, path, data)?;

    Ok(())
}
