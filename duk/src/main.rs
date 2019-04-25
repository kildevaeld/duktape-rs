extern crate colored;
extern crate duktape;
extern crate duktape_es2015;
extern crate env_logger;
extern crate log;
extern crate rustyline;
#[macro_use]
extern crate clap;
mod repl;

use duktape::prelude::*;
use duktape::vfs::physical::PhysicalFS;
fn main() -> DukResult<()> {
    env_logger::init();

    let matches = clap_app!(duk =>
        (@arg execute: -e --execute "")
        (@arg es2015: -z --es2015 "")
        (@arg compile: -c --compile "")
        (@arg input: "Input file")
    )
    .get_matches();

    let es6 = matches.is_present("es2015");
    let compile = matches.is_present("compile");

    let ctx = Context::new_with_env(Environment::from_env()?).unwrap();

    let mut require = Require::new().resolver(
        "file",
        file_resolver(
            PhysicalFS::new("/").unwrap(),
            ctx.env().cwd().to_string_lossy(),
        ),
    );

    if es6 || compile {
        require = duktape_es2015::register(&ctx, require)?;
    }

    require.build(&ctx).expect("require");

    if let Some(script) = matches.value_of("input") {
        if matches.is_present("execute") {
            if compile {
                let out = compile_js(&ctx, script)?;
                println!("{}", out);
                Ok(())
            } else {
                ctx.eval_main_script("", script).map(|_| ())
            }
        } else {
            if compile {
                let script = std::fs::read_to_string(std::path::Path::new(script))?;
                let out = compile_js(&ctx, &script)?;
                println!("{}", out);
                Ok(())
            } else {
                ctx.eval_main(script).map(|_| ())
            }
        }
    } else {
        return repl::run(&ctx, es6);
    }?;

    Ok(())
}

fn compile_js<'a>(ctx: &'a Context, source: &str) -> DukResult<&'a str> {
    ctx.require("es2015")?
        .prop("transform")
        .call::<_, Object>(source)?
        .get::<_, &str>("code")
}
