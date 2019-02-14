#[macro_export]
macro_rules! duk_error {
    ($msg: expr) => {
        return Err($crate::error::DukError::new(
            $crate::error::DukErrorCode::Error,
            $msg,
        ));
    };
}

#[macro_export]
macro_rules! duk_type_error {
    ($msg: expr) => {
        return Err($crate::error::DukError::new(
            $crate::error::DukErrorCode::Type,
            $msg,
        ));
    };
}

#[macro_export]
macro_rules! duk_reference_error {
    ($msg: expr) => {
        return Err($crate::error::DukError::new(
            $crate::error::DukErrorCode::Reference,
            $msg,
        ));
    };
}
