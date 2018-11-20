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

    let mut require = duktape_cjs::Builder::new();

    duktape_stdlib::register(&ctx, &mut require, duktape_stdlib::Modules::all());

    duktape_cjs::register(&ctx, require)?;

    duktape_stdlib::init_runtime(&ctx);

    let args = env::args().collect::<Vec<_>>();
    let (path, data) = if args.len() < 2 {
        return repl::run(&ctx);
    } else if args.len() == 3 && args[1] == "-e" {
        ("", args[2].as_bytes().to_vec())
    } else {
        let path = &args[1];
        let data = fs::read(path)?;
        (path.as_str(), data)
    };

    duktape_cjs::eval_main_script(&mut ctx, path, data)?;

    Ok(())
}
