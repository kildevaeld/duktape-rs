extern crate duktape_sys;
#[macro_use]
extern crate error_chain;
//#[cfg(feature = "serde")]
//extern crate serde;
extern crate typemap;

mod callable;
//mod class_builder;
mod context;
//#[cfg(feature = "serde")]
//mod duk_serde;
mod encoding;
pub mod error;
mod internal;
mod references;

pub use self::callable::{cb, Callable, CallableBoxed};
//pub use self::class_builder::*;
pub use self::context::{Context, Idx, Type};
pub use self::encoding::*;
pub use self::references::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
