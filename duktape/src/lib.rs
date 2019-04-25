#[macro_use]
extern crate bitflags;

#[macro_use]
mod macros;

mod argument_list;
mod array;
mod callable;
pub mod class;
// mod streams;
#[cfg(feature = "commonjs")]
pub mod commonjs;
mod context;
#[cfg(feature = "runtime")]
pub mod env;
mod error;
mod from_context;
mod function;
mod object;
mod privates;
mod property;
mod reference;
#[cfg(feature = "runtime")]
mod runtime;
#[cfg(feature = "serde")]
mod serialize;
mod to_context;
#[cfg(feature = "commonjs")]
pub use vfs;

pub mod types {
    pub use super::argument_list::*;
    pub use super::array::*;
    pub use super::function::*;
    pub use super::object::*;
    pub use super::reference::*;
}

pub mod prelude {
    pub use super::callable::*;
    pub use super::class;
    pub use super::class::ContextClassBuilder;
    #[cfg(feature = "commonjs")]
    pub use super::commonjs::prelude::*;
    pub use super::context::*;
    #[cfg(feature = "runtime")]
    pub use super::env::*;
    pub use super::error::*;
    pub use super::from_context::*;
    pub use super::macros::*;
    pub use super::property::*;
    pub use super::to_context::*;
    pub use super::types::*;

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
