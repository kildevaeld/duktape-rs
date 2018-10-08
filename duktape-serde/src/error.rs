use serde::ser;
use std::fmt::Display;

error_chain!{

    errors {
        Message(msg:String) {
            description("SerializationError")
            display("error: {}", msg)
        }
    }

}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        ErrorKind::Message(msg.to_string()).into()
    }
}
