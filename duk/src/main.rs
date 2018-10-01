extern crate duktape;
extern crate duktape_cjs;
use std::io::{self, Write};
use std::thread::sleep_ms;
use std::{env, fs};

struct Test;

impl Test {
    fn push_stdio(&self, ctx: &mut duktape::Context) {
        // let stdout = duktape::ClassBuilder::new();

        // stdout.method(
        //     "write",
        //     Box::new(|ctx| {
        //         let s: String = ctx.get(0)?;
        //         io::stdout().lock().write(s.as_bytes());
        //         Ok(0)
        //     }),
        // ).method(
        //     "flush",
        //     Box::new(|ctx| {
        //         let s: String = ctx.get(0)?;
        //         io::stdout().lock().write(s.as_bytes());
        //         Ok(0)
        //     }),
        // );

        ctx.push_object()
            .push(duktape::cb(
                1,
                Box::new(|ctx| {
                    let s: String = ctx.get(0)?;
                    io::stdout().lock().write(s.as_bytes());
                    Ok(0)
                }),
            )).put_prop_string(-2, "write")
            .push(duktape::cb(
                0,
                Box::new(|ctx| {
                    io::stdout().lock().flush();
                    Ok(0)
                }),
            )).put_prop_string(-2, "flush")
            .put_prop_string(-2, "stdout");
    }
}

impl duktape::Callable for Test {
    fn call(&self, ctx: &mut duktape::Context) -> duktape::error::Result<i32> {
        ctx.push_object();
        self.push_stdio(ctx);

        Ok(1)
    }
}

fn main() -> duktape::error::Result<()> {
    let mut ctx = duktape::Context::new().unwrap();

    let mut require = duktape_cjs::RequireBuilder::new();

    let m: Box<dyn duktape::Callable> = Box::new(Test {});

    require.module("io", m);

    duktape_cjs::register(&mut ctx, require);

    let args = env::args();

    if args.len() < 2 {
        println!("usage: duk <path>");
        return Ok(());
    }

    let path = &args.collect::<Vec<String>>()[1];
    let data = fs::read(path).unwrap();

    ctx.eval(data)?;
    ctx.pop(1).push_object();
    let r = duktape::Reference::new(&mut ctx, -1);
    let mut f = r.into_object();
    f.set("test", "Hello, World");
    f.set(
        "rapper",
        duktape::cb(
            1,
            Box::new(|ctx| {
                println!("Hello {}", ctx.get::<String>(0)?);
                Ok(0)
            }),
        ),
    );
    f.call("fn_name: T", ("Hello"))?;
    //ctx.pop(1);
    //println!("{}", ctx.dump());

    sleep_ms(10000);

    Ok(())
}
