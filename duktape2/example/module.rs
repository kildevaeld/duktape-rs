use duktape2::prelude::*;
use vfs::physical::PhysicalFS;

fn main() -> DukResult<()> {
    let ctx = Context::new().unwrap();

    Require::new()
        .env(Environment::from_env().unwrap())
        .resolver(
            "file",
            file_resolver(
                PhysicalFS::new("/").unwrap(),
                "/Users/rasmus/.marks/rust/duktape",
            ),
        )
        .build(&ctx)
        .expect("require");

    let parent = ctx
        .push(jsfunc(|_ctx: &Context| {
            //
            println!("parent ctor");
            Ok(0)
        }))
        .unwrap()
        .getp::<Function>()?;

    parent.set(
        "toString",
        jsfunc(|ctx: &Context| {
            ctx.push("Parent")?;
            Ok(1)
        }),
    )?;

    parent.set_prototype(&ctx.create::<Object>()?)?;

    let mut test = ctx.create_class(
        |_ctx: &Context, this: &class::Instance| {
            let ctor = this.get::<_, Object>("constructor")?;
            let parent = ctor.get::<_, Function>("__super__").unwrap();
            parent.call_ctx(this, ())?;
            println!(
                "Rapper 123 {:?}",
                ctor.get::<_, Reference>("__super__").unwrap()
            ); //.get::<_, Reference>("__super__"));
            Ok(0)
        },
        Some(parent.clone()),
    )?;

    test.set_name("Test");

    println!("Test {:?}", test.get::<_, &str>("name"));

    test.property(Property::build("greeting").value("Hello, World"))?;

    ctx.push_global_object()
        .getp::<Object>()?
        .set("Test", test)?;

    ctx.push_global_object()
        .getp::<Object>()?
        .set("Parent", parent)?;

    ctx.eval_main("./test.js")?.push();

    Ok(())
}
