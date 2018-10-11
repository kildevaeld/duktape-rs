use std::io;
use std::str;
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
        ReferenceError(message: String) {
            description("ReferenceError")
            display("Reference error: {}", message)
        }

        EvalError(message: String) {
            description("EvalError")
            display("Eval error: {}", message)
        }

        Error(message: String) {
            description("Error")
            display("Error: {}", message)
        }
    }

    foreign_links {
        Utf8(str::Utf8Error);
        Io(io::Error);
    }

}

macro_rules! err_impl {
    ($name: ident, $errtype: ident) => {
        pub fn $name<T: AsRef<str>>(msg: T) -> Error {
            ErrorKind::$errtype(msg.as_ref().to_owned()).into()
        }
    };
}

impl Error {
    err_impl!(type_err, TypeError);
    err_impl!(ref_err, ReferenceError);
    err_impl!(eval_err, EvalError);
    err_impl!(err, Error);
}
