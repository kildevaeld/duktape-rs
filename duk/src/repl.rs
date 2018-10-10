use duktape::prelude::*;
use duktape_cjs::error::Result;
use rustyline::error::ReadlineError;
use rustyline::{ColorMode, CompletionType, Config, EditMode, Editor};
use std::env;

pub fn run(ctx: &Context) -> Result<()> {
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

    loop {
        let readline = rl.readline("duk> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                match ctx.eval(&line) {
                    Err(e) => println!("  {:?}", e),
                    Ok(_) => {}
                };
                //println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                //println!("CTRL-C");
                break;
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
