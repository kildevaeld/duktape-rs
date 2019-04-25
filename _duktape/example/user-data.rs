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

    let greeter: Object = ctx.get_global_string("Greeter").construct(0)?.getp();
    println!("{}", greeter.call::<String>("greet", "eevee")?);

    Ok(())
}
