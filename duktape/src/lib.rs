extern crate duktape_sys;
#[macro_use]
extern crate error_chain;

mod callable;
mod context;
mod encoding;
pub mod error;
pub use self::callable::{cb, Callable};
pub use self::context::Context;
pub use self::encoding::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
