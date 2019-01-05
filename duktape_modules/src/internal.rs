use super::commonjs::build_require;
use super::error::Result;
use duktape::prelude::*;
use std::path::Path;
use std::str;

//pub fn push_require_function(ctx: &Context) -> Result<()> {}

pub fn push_module_object<'a, T: AsRef<Path>>(
    ctx: &'a Context,
    path: T,
    main: bool,
) -> Result<Object<'a>> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or(Path::new(""));
    let path_str = path.to_str();

    let module = ctx.create::<Object>()?;
    module
        .set("fileName", path_str)
        .set("dirName", parent.to_str())
        .set("id", path_str)
        .set("loaded", false)
        .set("require", ctx.get_global_string("require").getp::<Ref>()?)
        .set("exports", ctx.create::<Object>()?);

    if main {
        ctx.push_global_stash()
            .push(path_str)?
            .put_prop_string(-2, "main")
            .pop(1);
    }

    Ok(module)
}

pub fn eval_module<'a>(ctx: &'a duktape::Context, script: &[u8], module: &Object) -> Result<()> {
    let s = str::from_utf8(script)?;

    ctx.push_string("(function(exports,require,module,__filename,__dirname) {")
        .push_string(s)
        .push_string("\n})")
        .concat(3)?;

    ctx.push(module.get::<_, Ref>("fileName")?)?;
    ctx.compile(Compile::EVAL)?.call(0)?;

    let require = build_require(ctx, &module.get::<_, String>("id")?)?;

    Ok(ctx.getp::<Function>()?.call::<_, ()>((
        module.get::<_, Ref>("exports")?,
        require,
        module.clone(),
        module.get::<_, Ref>("fileName")?,
        module.get::<_, Ref>("dirName")?,
    ))?)
}
