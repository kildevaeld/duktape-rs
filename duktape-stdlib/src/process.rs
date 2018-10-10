use duktape::error::Result;
use duktape::prelude::*;
use std::env;

pub fn init_process(ctx: &Context) -> Result<()> {
    let global: Object = ctx.push_global_object().getp()?;

    let process: Object = ctx.create()?;

    process.set("cwd", |ctx: &Context| {
        let cwd = env::current_dir()?;
        ctx.push(cwd.to_str());
        Ok(1)
    });

    global.set("process", process);

    Ok(())
}
