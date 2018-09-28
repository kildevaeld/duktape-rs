mod duktape_ffi;
mod duktape_macros;

pub use self::duktape_ffi::*;
pub use self::duktape_macros::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
