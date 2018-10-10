# Duktape rust
**VERY MUCH WIP**

## Usage

```rust

extern crate duktape;

use duktape::prelude::*;

fn main() -> Result<()> {

    let mut ctx = Context::new();

    let greeting: String = ctx.push_global_object();
        // Push a function which takes 1 argument
        .push_function((1, |ctx: &Context| {
            if !ctx.is(Type::String) {
                // Throw an exception in js-land
                return Err(ErrorKind::TypeError("argument should be a string").into());
            }
            // Push a return value to the stack
            ctx.push(format!("Hello {}", ctx.get::<String>(0)))?;
            Ok(1)
        }))?
        // Attach the fn on the global object with the name 'fn'
        .set_prop_string(-2, "fn")?
        // remove the global object from the "stack"
        .pop(1); 
        // Call the rust closure from js
        .eval("fn('me')")?
        // Get and  pop the return value from the stack
        .getp()?;

    assert_eq(&greeting, "Hello me");

    Ok(())
}


```

```rust

extern crate duktape;
use duktape::error::Result;
use duktape::prelude::*;

fn main() -> Result<()> {
    let mut ctx = Context::new()?;

    let mut builder = class::build();

    let global: Object = ctx.push_global_object().getp()?;

    builder.method(
        "greet",
        (1, |ctx: &Context, this: &mut class::Instance| {
            let name = ctx.get::<String>(0)?;
            ctx.push(format!("Hello {}", name))?;
            Ok(1)
        }),
    );

    global.set("Greeter", builder);

    let greeting: String = ctx
        .eval(
            r#"
    
    var greeter = new Greeter();

    var greeting = greeter.greet('me');
    greeting + '!';
    "#,
        )?
        .get(-1)?;

    assert_eq!(greeting, "Hello me!");
    println!("{}", greeting);

    let greeter: Object = ctx.get_global_string("Greeter").construct(0)?.getp()?;
    println!("{}", greeter.call::<_, _, String>("greet", "eevee")?);

    Ok(())
}



```