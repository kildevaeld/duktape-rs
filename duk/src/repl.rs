use colored::*;
use duktape::prelude::*;
use duktape_modules::error::Result;
use duktape_modules::CJSContext;
use rustyline::error::ReadlineError;
use rustyline::{ColorMode, CompletionType, Config, EditMode, Editor};
use std::env;

fn print_help() {}

pub fn run(ctx: &Context, es6: bool) -> Result<()> {
    let require: Object = ctx.get_global_string("require").getp()?;

    require.set(
        b"\xFFmoduleId",
        format!("{}/___repl.js", env::current_dir()?.to_str().unwrap()),
    );

    let config = Config::builder()
        .edit_mode(EditMode::Vi)
        .completion_type(CompletionType::List)
        .color_mode(ColorMode::Enabled)
        .history_ignore_dups(true)
        .build();

    let mut rl = Editor::<()>::with_config(config);

    if rl.load_history("duk_history.txt").is_err() {
        println!("No previous history.");
    }

    let mut next_should_close = false;

    loop {
        let readline = rl.readline("duk> ");

        match readline {
            Ok(line) => {
                next_should_close = false;

                match line.as_str() {
                    ".exit" => break,
                    ".stack" => {
                        println!("{:?}", ctx);
                        continue;
                    }
                    ".refs" => {
                        // ctx.push_global_stash().;

                        continue;
                    }
                    ".help" => {
                        print_help();
                        continue;
                    }
                    _ => {}
                }

                rl.add_history_entry(line.as_ref());

                let source = if es6 {
                    ctx.require("es2015")?
                        .call::<_, _, Object>("transform", line.as_str())?
                        .get::<_, &str>("code")?
                } else {
                    line.as_str()
                };

                match ctx.eval(source.replace("\'use strict\';", "").trim()) {
                    Err(e) => println!("  {:?}", e),
                    Ok(_) => {
                        let re = ctx.getp::<Ref>()?;
                        let s = match re.get_type() {
                            Type::Boolean | Type::Number => format!("{}", re.to_string().yellow()),
                            Type::Null => format!("{}", re.to_string().white()),
                            Type::String => format!("'{}'", re.to_string().green()),
                            _ => format!("{}", re.to_string().bright_black()),
                        };
                        println!("{}", s);
                    }
                };
            }
            Err(ReadlineError::Interrupted) => {
                if next_should_close {
                    break;
                }
                next_should_close = true;
                println!("(To exit, press ^C again or type .exit)");
            }
            Err(ReadlineError::Eof) => {
                //println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("duk_history.txt").unwrap();

    Ok(())
}
