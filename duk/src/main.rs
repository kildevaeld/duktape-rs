extern crate duktape;
extern crate duktape_cjs;
extern crate duktape_stdlib;

use duktape::prelude::*;
use std::{env, fs};
extern crate env_logger;
extern crate log;

fn main() -> duktape_cjs::error::Result<()> {
    env_logger::init();

    let mut ctx = Context::new().unwrap();

    let mut require = duktape_cjs::RequireBuilder::new();
    duktape_stdlib::init(&mut require);

    duktape_cjs::register(&mut ctx, require)?;

    let args = env::args();

    if args.len() < 2 {
        println!("usage: duk <path>");
        return Ok(());
    }

    let path = &args.collect::<Vec<String>>()[1];
    let data = fs::read(path).unwrap();
    ctx.push_global_stash();

    ctx.pop(1);
    let module = duktape_cjs::eval_main_script(&mut ctx, path, data)?;
    println!("{}", module);
    Ok(())
}
