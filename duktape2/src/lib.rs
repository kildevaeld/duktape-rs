#[macro_use]
extern crate bitflags;

#[macro_use]
mod macros;

mod argument_list;
mod array;
mod callable;
// pub mod class;
#[cfg(feature = "commonjs")]
pub mod commonjs;
mod context;
mod error;
mod from_context;
mod function;
mod object;
mod property;
mod reference;
// mod streams;
mod to_context;

mod privates;

pub mod types {
    pub use super::argument_list::*;
    pub use super::array::*;
    pub use super::object::*;
    pub use super::reference::*;
    pub use super::function::*;
}

pub mod prelude {
    pub use super::callable::*;
    // pub use super::class;
    #[cfg(feature = "commonjs")]
    pub use super::commonjs::*;
    pub use super::context::*;
    pub use super::error::*;
    pub use super::from_context::*;
    pub use super::macros::*;
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
