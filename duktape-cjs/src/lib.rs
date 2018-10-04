#[macro_use]
extern crate error_chain;
extern crate duktape;
#[macro_use]
extern crate lazy_static;
extern crate regex;

//mod class_builder;
//pub mod class;
mod commonjs;
pub mod error;
mod types;

//pub use self::class::*;
pub use self::commonjs::RequireBuilder;
pub use self::types::*;

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
        //.push(Box::new(commonjs::Require::new()) as Box<dyn duktape::Callable>)
        .put_prop_string(-2, "require");

    ctx.pop(1);

    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
