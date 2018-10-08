extern crate duktape;
extern crate duktape_sys;
extern crate serde;
#[macro_use]
extern crate error_chain;

mod error;
mod serializer;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
