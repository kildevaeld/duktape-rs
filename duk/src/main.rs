extern crate colored;
extern crate duktape;
extern crate duktape_cjs;
extern crate duktape_es2015;
extern crate duktape_stdlib;
extern crate env_logger;
extern crate log;
extern crate rustyline;
#[macro_use]
extern crate clap;
mod repl;

use duktape::prelude::*;
use duktape_cjs::CJSContext;

fn main() -> duktape_cjs::error::Result<()> {
    env_logger::init();

    let matches = clap_app!(duk =>
        (@arg execute: -e --execute "")
        (@arg es2015: -z --es2015 "")
        (@arg input: "Input file")
    )
    .get_matches();

    let es6 = matches.is_present("es2015");

    let ctx = Context::new().unwrap();

    let mut require = duktape_cjs::Builder::new();

    duktape_stdlib::register(&ctx, &mut require, duktape_stdlib::Modules::all());

    if es6 {
        duktape_es2015::register(&ctx, &mut require);
    }

    duktape_cjs::register(&ctx, require)?;
    duktape_stdlib::init_runtime(&ctx);

    if let Some(script) = matches.value_of("input") {
        if matches.is_present("execute") {
            ctx.eval_main_script("", script)
        } else {
            ctx.eval_main(script)
        }
    } else {
        return repl::run(&ctx, es6);
    }?;

    Ok(())
}
