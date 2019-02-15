use duktape2::prelude::*;

fn main() {
    let ctx = Context::new().unwrap();

    Require::build().build(&ctx);

    ctx.eval("require('./hello')");
}
