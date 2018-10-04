extern crate duktape;
extern crate duktape_cjs;
extern crate duktape_stdlib;

use duktape::prelude::*;
use std::io::{self, Write};
use std::{env, fs};
extern crate env_logger;
extern crate log;

struct Test;

impl Test {
    fn push_stdio(&self, ctx: &mut duktape::Context) {
        ctx.push_object()
            .push(duktape::cb(
                1,
                Box::new(|ctx| {
                    let s: String = ctx.get(0)?;
                    io::stdout().lock().write(s.as_bytes()).unwrap();
                    Ok(0)
                }),
            ))
            .put_prop_string(-2, "write")
            .push(duktape::cb(
                0,
                Box::new(|_ctx| {
                    io::stdout().lock().flush().unwrap();
                    Ok(0)
                }),
            ))
            .put_prop_string(-2, "flush")
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

#[derive(Debug)]
struct UserData {
    pub name: String,
}

struct UserDataKey;

impl duktape::Key for UserDataKey {
    type Value = UserData;
}

impl duktape::Key for UserData {
    type Value = String;
}

impl Drop for UserData {
    fn drop(&mut self) {
        println!("me {}", "dropped");
    }
}

fn main() -> duktape::error::Result<()> {
    env_logger::init();

    let mut ctx = duktape::Context::new().unwrap();

    let mut require = duktape_cjs::RequireBuilder::new();
    duktape_stdlib::init(&mut require);

    duktape_cjs::register(&mut ctx, require);

    let args = env::args();

    if args.len() < 2 {
        println!("usage: duk <path>");
        return Ok(());
    }

    let path = &args.collect::<Vec<String>>()[1];
    let data = fs::read(path).unwrap();

    let mut builder = duktape::class::Builder::default();

    builder
        .constructor(|_ctx, this| {
            this.data_mut().insert::<UserDataKey>(UserData {
                name: "This is my name".to_owned(),
            });
            println!("constructor");
            Ok(0)
        })
        .method("hello", |_ctx, this| {
            println!("Hello, Swerk {:?}", this.data().get::<UserDataKey>());
            Ok(0)
        });

    let mut global: Object = ctx.push_global_object().getp()?;
    global.set("TestClass", builder);

    ctx.eval(data)?;

    //let mut f = ctx.pop(1).push_object().get::<Object>(-1)?;

    // f.set("test", "Hello, World").set(
    //     "rapper",
    //     duktape::cb(
    //         2,
    //         Box::new(|ctx| {
    //             if !ctx.is(duktape::Type::String) {}
    //             println!("Hello {}", ctx.get::<String>(0)?);
    //             ctx.push(format!("Back {}", ctx.get::<i32>(1)? + 1));
    //             Ok(1)
    //         }),
    //     ),
    // );
    // println!("{:?}", ctx);
    // let ret: String = f.call("rapper", ("Hello", 2, 23))?;

    // println!("{}", ret);

    // let o: Object = ctx.push_global_object().getp()?;

    // let ret: i32 = o
    //     .get::<&str, Object>("Math")?
    //     .call("min", ("rapraprap", 20))?;
    // println!("result {}", ret);

    // let array: Array = ctx.push_array().getp()?;
    // array.push("Hello").push(2).push("fixed");

    // for a in array.iter() {
    //     println!("{}", a);
    // }

    // ctx.push(vec!["Hello", "Array", "rerere"]);
    // println!("{}", ctx.dump());

    //sleep_ms(10000);

    Ok(())
}
