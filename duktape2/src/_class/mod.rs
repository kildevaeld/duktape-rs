//mod builder;
mod builder2;
mod method;

pub use self::builder2::*;
pub use self::method::{Instance, Method};

pub fn new<'a>() -> ClassBuilder<'a> {
    ClassBuilder::default()
}

// #[cfg(test)]
// pub mod tests {

//     use super::super::context::Context;
//     use super::super::object::Object;
//     use super::method::Instance;
//     #[test]
//     fn class_builder() {
//         let ctx = Context::new().unwrap();

//         let mut b = super::build();

//         b.method("testMethodNoArg", |ctx: &Context, this: &mut Instance| {
//             ctx.push_string("Hello, World!");
//             Ok(1)
//         })
//         .method(
//             "testMethodArg",
//             (1, |ctx: &Context, this: &mut Instance| Ok(0)),
//         );

//         // ctx.push_class(b).unwrap();
//         // ctx.construct(0).unwrap();

//         // let out = ctx.getp::<Object>().unwrap();

//         // let greeting = out.call::<_, _, String>("testMethodNoArg", ()).unwrap();
//         // assert_eq!(greeting, "Hello, World!");
//     }
// }
