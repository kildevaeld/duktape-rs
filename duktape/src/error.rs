use std::error::Error;
use std::fmt;
use std::io;
use std::result::Result;
use std::str;

pub type DukResult<T> = Result<T, DukError>;

#[derive(PartialEq, Debug, Clone)]
pub enum DukErrorCode {
    None,      /* no error (e.g. from duk_get_error_code()) */
    Error,     /* Error */
    Eval,      /* EvalError */
    Range,     /* RangeError */
    Reference, /* ReferenceError */
    Syntax,    /* SyntaxError */
    Type,      /* TypeError */
    Uri,
}

#[derive(Debug)]
pub struct DukError {
    msg: Option<String>,
    code: DukErrorCode,
    inner: Option<Box<dyn Error + 'static>>,
}

impl DukError {
    pub fn new<S: AsRef<str>>(code: DukErrorCode, msg: S) -> DukError {
        DukError {
            code,
            msg: Some(msg.as_ref().to_owned()),
            inner: None,
        }
    }

    pub fn with<E: Error + 'static>(source: E) -> DukError {
        DukError {
            code: DukErrorCode::Error,
            msg: None,
            inner: Some(Box::new(source)),
        }
    }

    pub fn code(&self) -> &DukErrorCode {
        &self.code
    }
}

impl fmt::Display for DukError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}Error", self.code)?;

        if let Some(m) = &self.inner {
            write!(f, ": {}", m)?;
        } else if let Some(m) = &self.msg {
            write!(f, ": {}", m)?;
        }

        Ok(())
    }
}

macro_rules! err_impl {
    ($name: ident, $errtype: ident) => {
        pub fn $name<S: AsRef<str>>(msg: S) -> DukError {
            DukError::new(DukErrorCode::$errtype, msg.as_ref().to_string())
        }
    };
}

impl DukError {
    err_impl!(type_err, Type);
    err_impl!(ref_err, Reference);
    err_impl!(eval_err, Eval);
    err_impl!(err, Error);
}

impl Error for DukError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.inner {
            Some(m) => Some(m.as_ref()),
            None => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InsufficientMemory;

impl fmt::Display for InsufficientMemory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InsufficientMemory")
    }
}

impl Error for InsufficientMemory {}

#[derive(Debug, PartialEq)]
pub struct InvalidIndex;

impl fmt::Display for InvalidIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InvalidIndex")
    }
}

impl Error for InvalidIndex {}

impl From<InsufficientMemory> for DukError {
    fn from(error: InsufficientMemory) -> DukError {
        DukError::with(error)
    }
}

impl From<InvalidIndex> for DukError {
    fn from(error: InvalidIndex) -> DukError {
        DukError::with(error)
    }
}

impl From<str::Utf8Error> for DukError {
    fn from(error: str::Utf8Error) -> DukError {
        DukError::with(error)
    }
}

impl From<io::Error> for DukError {
    fn from(error: io::Error) -> DukError {
        DukError::with(error)
    }
}


#[cfg(feature = "serde")]
use serde::de;

#[cfg(feature = "serde")]
impl de::Error for DukError {
    #[cold]
    fn custom<T: fmt::Display>(msg: T) -> DukError {
        DukError::new(DukErrorCode::Type, msg.to_string())
        // make_error(msg.to_string())
    }

    #[cold]
    fn invalid_type(unexp: de::Unexpected, exp: &de::Expected) -> Self {
        if let de::Unexpected::Unit = unexp {
            DukError::custom(format_args!("invalid type: null, expected {}", exp))
        } else {
            DukError::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
        }
    }
}