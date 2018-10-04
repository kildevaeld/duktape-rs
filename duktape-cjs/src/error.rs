use duktape::error;
use std::io;
use std::str;
error_chain!{
    foreign_links {
        Io(io::Error);
        Utf8(str::Utf8Error);
    }

    links {
        Duktape(error::Error, error::ErrorKind);
    }
}
