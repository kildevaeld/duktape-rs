extern crate duktape;
extern crate duktape_cjs;

mod fs;
mod io;

use duktape::prelude::*;

pub fn init(builder: &mut duktape_cjs::RequireBuilder) {
    builder
        .module("io", |ctx: &mut Context| {
            return io::init_io(ctx);
        })
        .module("fs", |ctx: &mut Context| {
            return fs::init_fs(ctx);
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
