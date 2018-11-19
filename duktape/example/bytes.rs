extern crate duktape;
use duktape::error::Result;
use duktape::prelude::*;

fn main() -> Result<()> {
    let ctx = Context::new()?;
    let bs = b"Hello, World";
    ctx.push(&bs[..])?;

    let bs = ctx.get_bytes(-1);
    println!("{:?}", bs);

    Ok(())
}
