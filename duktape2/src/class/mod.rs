mod builder;
mod method;

pub use self::builder::*;
pub use self::method::{Instance, Method};

pub fn build<'a>() -> Builder<'a> {
    Builder::default()
}
