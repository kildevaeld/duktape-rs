extern crate duktape;

struct Test;

impl duktape::Callable for Test {
    fn call(&self, ctx: &mut duktape::Context) -> duktape::error::Result<i32> {
        println!("{}", ctx.dump());
        ctx.push("Hello, World")?;
        //println!("Hello, World");
        Ok(1)
    }
}

fn main() -> duktape::error::Result<()> {
    let mut ctx = duktape::Context::new().unwrap();

    ctx.eval("2+2")?;
    ctx.push(2 + 2)?;
    ctx.push("Hello, World")?;

    let b: Box<dyn duktape::Callable> = Box::new(Test {});
    ctx.push_global_object();
    //ctx.push(b)?;
    ctx.push(duktape::cb(
        1,
        Box::new(|ctx| {
            ctx.push(ctx.get::<String>(-1)?)?;
            Ok(1)
        }),
    ))?;

    ctx.put_prop_string(-2, "fn");
    //println!("{}", ctx.dump());
    ctx.eval("fn('Hellom World')")?;
    //ctx.eval("fn(\"Hello\")")?;
    println!("{}", ctx.dump());
    //println!("{}", ctx.dump());
    //let s: String = ctx.get(-1)?;

    //println!("{}", s);

    Ok(())
}
