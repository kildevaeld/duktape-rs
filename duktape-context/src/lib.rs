extern crate duktape_sys;
#[macro_use]
extern crate error_chain;
extern crate typemap;

mod callable;
mod class;
mod context;
pub mod error;
mod privates;

pub use self::callable::Callable;
pub use self::class::*;
pub use self::context::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
