use duktape2::prelude::*;

fn main() {
    let ctx = Context::new().unwrap();

    ctx.eval("var test = 'Hello, World'").unwrap();

    println!("{:?}", ctx);
}
