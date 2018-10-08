extern crate duktape_sys;
#[macro_use]
extern crate error_chain;
extern crate typemap;

mod callable;
pub mod class;
mod context;
pub mod error;
mod privates;
pub mod types;

pub use self::callable::Callable;
pub use self::context::*;

pub mod prelude {
    pub use super::callable::Callable;
    pub use super::context::*;
    pub use super::types::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
