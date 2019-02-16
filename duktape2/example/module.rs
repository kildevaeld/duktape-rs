use duktape2::commonjs::file_resolver;
use duktape2::prelude::*;
use vfs::physical::PhysicalFS;

fn main() {
    let ctx = Context::new().unwrap();

    Require::build()
        .resolver("file", file_resolver(PhysicalFS::new("/").unwrap()))
        .build(&ctx);

    ctx.eval("require('./hello')").unwrap();
}
