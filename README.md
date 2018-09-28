# Duktape rust


## Usage

```rust

extern crate duktape;

use duktape::*;

fn main() -> Result<()> {

    let mut ctx = Context::new();

    ctx.push_global_object();
    ctx.push(cb(1, Box::new(|ctx| {
        // Push a return value to the stack
        ctx.push(format!("Hello {}", ctx.get::<String>(0)));
        Ok(1)
    })))?;
    // Attach the fn on the global object with the name fn
    ctx.set_prop_string(-2, "fn")?;

    ctx.pop(); // remove the global object from the "stack"

    // Call the rust closure from js
    ctx.eval("fn('me')")?;

    // Get and  pop the return value from the stack
    let greeting: String = ctx.getp();

    assert_eq(&greeting, "Hello me");

    Ok(())
}


```