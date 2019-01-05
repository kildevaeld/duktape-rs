
pub(crate) static UTILS: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/utils.js"));
pub(crate) static RUNTIME: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/runtime.js"));
pub(crate) static FS: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/fs.js"));
pub(crate) static IO_JS: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/io.js"));
pub(crate) static HTTP: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/http.js"));
