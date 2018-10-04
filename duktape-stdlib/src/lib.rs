extern crate duktape;
extern crate duktape_cjs;

mod io;

use duktape::prelude::*;

pub fn init(builder: &mut duktape_cjs::RequireBuilder) {
    builder.module("io", |ctx: &mut Context| {
        return io::init_io(ctx);
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
