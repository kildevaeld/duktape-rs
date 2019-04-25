extern crate duktape_sys;
#[macro_use]
extern crate error_chain;
extern crate typemap;
#[macro_use]
extern crate bitflags;
#[cfg(feature = "value")]
extern crate value;
mod callable;
pub mod class;
mod context;
pub mod error;
mod macros;
mod privates;
pub mod types;

pub use self::callable::Callable;
pub use self::context::*;
pub use self::macros::*;
pub use self::typemap::Key;

pub mod prelude {
    pub use super::callable::Callable;
    pub use super::class;
    pub use super::context::*;
    pub use super::error::Error as DukError;
    pub use super::error::ErrorKind as DukErrorKind;
    pub use super::error::Result as DukResult;
    pub use super::types::*;
    pub use super::macros::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
