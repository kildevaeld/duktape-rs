#![feature(concat_idents)]
extern crate duktape_sys;
#[macro_use]
extern crate error_chain;
extern crate typemap;
#[macro_use]
extern crate log;

mod callable;
mod context;
mod encoding;
pub mod error;
mod privates;
// Types
mod argument_list;
mod array;
pub mod class;
mod function;
mod object;
mod reference;

pub use self::callable::{cb, Callable, CallableBoxed};
pub use self::context::{Context, Idx, Type};
pub use self::encoding::*;
pub use typemap::Key;

// Expose types;
pub mod types {
    pub use super::argument_list::*;
    pub use super::array::Array;
    pub use super::function::*;
    pub use super::object::*;
    pub use super::reference::*;
}

pub mod prelude {
    pub use super::context::*;
    pub use super::error::*;
    pub use super::types::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
