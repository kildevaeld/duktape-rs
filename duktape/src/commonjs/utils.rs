use super::super::context::{Compile, Context};
use super::super::error::DukResult;
use super::super::from_context::FromDuktapeContext;
use super::super::function::*;
use super::super::object::*;
use super::super::reference::Reference;
use super::super::to_context::ToDuktapeContext;
use super::require::Require;
use super::traits::*;
use std::path::Path;
use std::str;


pub(crate) static REQUIRE_JS: &'static str = include_str!("./require.js");

pub(crate) fn build_require<'a>(ctx: &'a Context, module_id: &str) -> DukResult<Function<'a>> {
    let function: Object = ctx.push_function(Require {}).getp()?;

    let mut stash: Object = ctx.push_global_stash().getp()?;
    stash = stash.get(KEY)?;

    function
        .set(MODULE_ID_KEY, module_id)?
        .set("cache", stash.get::<_, Reference>("cache")?)?
        .set("main", stash.get::<_, Reference>("main")?)?;

    let requirejs: Function = ctx
        .push_string(REQUIRE_JS)
        .push_string("require.js")
        .compile(Compile::EVAL)?
        .call(0)?
        .getp()?;

    let function = requirejs.call::<_, Function>(function);

    match function {
        Ok(mut ret) => {
            ret.set_name("require");
            Ok(ret)
        }
        Err(e) => Err(e),
    }
}

pub fn push_module_object<'a, T: AsRef<Path>>(
    ctx: &'a Context,
    path: T,
    main: bool,
) -> DukResult<Object<'a>> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or(Path::new(""));
    let path_str = path.to_str();

    let module = ctx.create::<Object>()?;
    module
        .set("fileName", path_str)?
        .set("dirName", parent.to_str())?
        .set("id", path_str)?
        .set("loaded", false)?
        .set(
            "require",
            ctx.get_global_string("require").getp::<Reference>()?,
        )?
        .set("exports", ctx.create::<Object>()?)?;

    if main {
        ctx.push_global_stash()
            .push(path_str)?
            .put_prop_string(-2, "main")
            .pop(1);
    }

    Ok(module)
}

pub fn eval_module<'a>(ctx: &'a Context, script: &[u8], module: &Object) -> DukResult<()> {
    let s = str::from_utf8(script)?;

    ctx.push_string("(function(exports,require,module,__filename,__dirname) {")
        .push_string(s)
        .push_string("\n})")
        .concat(3)?;

    ctx.push(module.get::<_, Reference>("fileName")?)?;
    ctx.compile(Compile::EVAL)?.call(0)?;

    let require = build_require(ctx, &module.get::<_, String>("id")?)?;

    Ok(ctx.getp::<Function>()?.call::<_, ()>((
        module.get::<_, Reference>("exports")?,
        require,
        module.clone(),
        module.get::<_, Reference>("fileName")?,
        module.get::<_, Reference>("dirName")?,
    ))?)
}
