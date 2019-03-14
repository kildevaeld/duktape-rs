use duktape2::prelude::*;
use vfs::physical::PhysicalFS;

fn main() {
    let ctx = Context::new().unwrap();

    Require::new()
        .resolver(
            "file",
            file_resolver(
                PhysicalFS::new("/").unwrap(),
                "/Users/rasmus/.marks/rust/duktape",
            ),
        )
        .build(&ctx)
        .expect("require");

    //ctx.eval("require('./test')").expect("require('test')");

    ctx.eval_main("./test.js").unwrap().push();

    println!("{:?}", ctx);
}
