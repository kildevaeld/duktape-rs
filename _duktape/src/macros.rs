#[macro_export]
macro_rules! duk_error {
    ($msg: expr) => {
        return Err($crate::error::ErrorKind::Error($msg.to_owned()).into());
    };
}

#[macro_export]
macro_rules! duk_type_error {
    ($msg: expr) => {
        return Err($crate::error::ErrorKind::TypeError($msg.to_owned()).into());
    };
}

#[macro_export]
macro_rules! duk_reference_error {
    ($msg: expr) => {
        return Err($crate::error::ErrorKind::ReferenceError($msg.to_owned()).into());
    };
}
