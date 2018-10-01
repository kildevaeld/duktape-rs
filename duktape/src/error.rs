use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::ser;

error_chain!{
    errors {
        Unknown {
            description("unknown error")
            display("unknown error")
        }
        InsufficientMemory {
            description("Insufficient Memory")
            display("Insufficient Memory")
        }
        TypeError(message: String) {
            description("TypeError")
            display("Type error: {}", message)
        }
    }
}

#[cfg(feature = "serde")]
impl ser::Error for Error {
    #[cold]
    fn custom<T: Display>(msg: T) -> Error {
        ErrorKind::Unknown.into()
    }
}
