use std::io;
use std::str;
use std::fmt;

pub enum DukErrorKind<'a> {
    InsufficientMemory,
    Type(&'a str),
    Reference(&'a str),
    Eval(&'a str),
    Syntax(&'a str),
    Uri(&'a str),
    Error(&'a str),
    Range(&'a str),
}

pub struct DukError<'a> {
    kind:DukErrorKind<'a>,
}

impl<'a> DukError<'a> {
    pub fn new(kind: DukErrorKind<'a>) -> DukError<'a> {
        DukError{
            kind
        }
    }

    pub fn kind(&self) -> &DukErrorKind {
        &self.kind
    }
}

impl<'a> fmt::Display for DukError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "err")
    }
}

macro_rules! err_impl2 {
    ($name: ident, $errtype: ident) => {
        pub fn $name<'b>(msg: &'b str) -> DukError<'b> {
            DukError::new(DukErrorKind::$errtype(msg))
        }
    };
}

impl<'a> DukError<'a> {
    err_impl2!(type_err, Type);
    err_impl2!(ref_err, Reference);
    err_impl2!(eval_err, Eval);
    err_impl2!(err, Error);
}

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
