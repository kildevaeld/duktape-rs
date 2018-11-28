use duktape::error::Result;
use duktape::prelude::*;
use std::env;

pub fn init_process(ctx: &Context) -> Result<()> {
    let global: Object = ctx.push_global_object().getp()?;

    let process: Object = ctx.create()?;

    process.set("cwd", |ctx: &Context| {
        let cwd = env::current_dir()?;
        ctx.push(cwd.to_str())?;
        Ok(1)
    });

    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    process.set("platform", platform);

    global.set("process", process);

    Ok(())
}
