#[macro_use]
extern crate error_chain;
extern crate duktape;
#[macro_use]
extern crate lazy_static;
extern crate duktape_sys;
extern crate regex;

//mod class_builder;
//pub mod class;
mod commonjs;
pub mod error;
mod types;

//pub use self::class::*;
pub use self::commonjs::RequireBuilder;
pub use self::types::*;

use duktape::types::{ArgumentList, Function, Object, Reference};
use duktape::Deserialize;
use std::fs;
use std::path::Path;
use std::str;

pub fn register(ctx: &mut duktape::Context, builder: RequireBuilder) -> bool {
    ctx.push_global_stash();
    if ctx.has_prop_string(-1, KEY) {
        return false;
    }

    ctx.push_bare_object()
        .push_bare_object()
        .put_prop_string(-2, "cache")
        .put_prop_string(-2, KEY);

    ctx.pop(1);

    ctx.push_global_object()
        .push(builder.build())
        .put_prop_string(-2, "require");

    ctx.pop(1);

    true
}

pub fn eval_main<'a, T: AsRef<Path>>(
    ctx: &'a mut duktape::Context,
    path: T,
) -> error::Result<Object> {
    let path = path.as_ref();
    let content = fs::read(path)?;
    eval_main_script(ctx, path, content)
}

pub fn eval_main_script<'a, T: AsRef<Path>, S: AsRef<[u8]>>(
    ctx: &'a mut duktape::Context,
    path: T,
    script: S,
) -> error::Result<Object> {
    let path = path.as_ref();
    let parent = path.parent().unwrap();

    let mut module = ctx.create::<Object>()?;
    module
        .set("fileName", path.clone().to_str())
        .set("dirName", parent.to_str())
        .set("id", path.to_str())
        .set("loaded", false)
        .set(
            "require",
            ctx.get_global_string("require").getp::<Reference>()?,
        )
        .set("exports", ctx.create::<Object>()?);

    eval_module(ctx, script.as_ref(), &mut module)?;

    Ok(module)
}

fn eval_module<'a>(
    ctx: &'a duktape::Context,
    script: &[u8],
    object: &mut duktape::types::Object,
) -> error::Result<()> {
    let s = str::from_utf8(script)?;

    ctx.push("(function(exports,require,module,__filename,__dirname) {")
        .push(s)
        .push("\n})")
        .concat(3)?;

    ctx.push(object.get::<_, Reference>("fileName")?);
    ctx.compile(duktape_sys::DUK_COMPILE_EVAL)?.call(0)?;

    Ok(ctx.getp::<Function>()?.call::<_, ()>((
        object.get::<_, Reference>("exports")?,
        object.get::<_, Reference>("require")?,
        object.clone(),
        object.get::<_, Reference>("fileName")?,
        object.get::<_, Reference>("dirName")?,
    ))?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
