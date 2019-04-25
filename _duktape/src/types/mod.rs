mod argument_list;
mod array;
mod from_duktape;
mod function;
mod object;
mod reference;
mod to_duktape;

#[derive(PartialEq, Debug)]
pub enum Type {
    Undefined,
    Null,
    String,
    Boolean,
    Number,
    Object,
    Array,
    Function,
    Buffer,
}

pub use self::argument_list::*;
pub use self::array::*;
pub use self::from_duktape::*;
pub use self::function::*;
pub use self::object::*;
pub use self::reference::*;
pub use self::to_duktape::*;
