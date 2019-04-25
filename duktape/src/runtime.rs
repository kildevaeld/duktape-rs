use super::callable::*;
use super::context::*;
use super::error::*;
use super::types::object::*;
use super::types::reference::*;

static RUNTIME_JS: &'static [u8] = include_bytes!("../userland/dist/runtime.js");
static POLYFILL_JS: &'static [u8] = include_bytes!("../userland/dist/polyfill.js");

pub(crate) fn init_runtime(ctx: &Context) -> DukResult<()> {
    ctx.compile_string(POLYFILL_JS, Compile::EVAL)?;
    ctx.call(0)?.pop(1);

    create_process(ctx)?;

    ctx.compile_string(RUNTIME_JS, Compile::EVAL)?;
    ctx.call(0)?.pop(1);

    Ok(())
}

fn create_process(ctx: &Context) -> DukResult<()> {
    let env = ctx.env();
    let process = ctx.create::<Object>()?;
    let empty = Vec::<String>::new();

    process.set("cwd", env.cwd().to_str().unwrap_or(""))?.set(
        "args",
        match env.args() {
            Some(s) => s,
            None => &empty,
        },
    )?;

    let stdout = ctx.create::<Object>()?;

    stdout.set(
        "write",
        jsfunc((1, |ctx: &Context| {
            let pipes = ctx.env_mut().pipes_mut();
            let arg = match ctx.get_type(0) {
                Type::String => ctx.get_string(0)?.as_bytes(),
                Type::Buffer => ctx.get_bytes(0)?,
                _ => return duk_type_error!(format!("invalid type {:?}", ctx.get_type(0))),
            };

            let size = pipes.stdout_mut().write(arg)?;
            ctx.push_uint(size as u32);

            Ok(1)
        })),
    )?;

    process.set("stdout", stdout)?;

    let stderr = ctx.create::<Object>()?;

    stderr.set(
        "write",
        jsfunc((1, |ctx: &Context| {
            let pipes = ctx.env_mut().pipes_mut();
            let arg = match ctx.get_type(0) {
                Type::String => ctx.get_string(0)?.as_bytes(),
                Type::Buffer => ctx.get_bytes(0)?,
                _ => return duk_type_error!(format!("invalid type {:?}", ctx.get_type(0))),
            };

            let size = pipes.stderr_mut().write(arg)?;
            ctx.push_uint(size as u32);

            Ok(1)
        })),
    )?;

    process.set("stderr", stderr)?;

    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    process.set("platform", platform)?;

    ctx.push_global_object();
    process.push();
    ctx.put_prop_string(-2, "process");

    Ok(())
}
