#[macro_use]
extern crate bitflags;

#[macro_use]
mod macros;
mod callable;
#[cfg(feature = "types")]
pub mod class;
#[cfg(feature = "commonjs")]
pub mod commonjs;
mod context;
#[cfg(feature = "runtime")]
pub mod env;
mod error;
mod from_context;
mod privates;
mod property;
#[cfg(feature = "runtime")]
mod runtime;
#[cfg(feature = "serde")]
mod serialize;
mod to_context;
#[cfg(feature = "commonjs")]
pub use vfs;

#[cfg(feature = "types")]
pub mod types;

pub mod prelude {
    pub use super::callable::*;
    #[cfg(feature = "types")]
    pub use super::class;
    #[cfg(feature = "types")]
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
    #[cfg(feature = "types")]
    pub use super::types::*;

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
