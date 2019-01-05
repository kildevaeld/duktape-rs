extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    // let bindings = bindgen::Builder::default()
    //     // The input header we would like to generate
    //     // bindings for.
    //     .header("wrapper.h")
    //     // Finish the builder and generate the bindings.
    //     .generate()
    //     // Unwrap the Result and panic on failure.
    //     .expect("Unable to generate bindings");

    // // Write the bindings to the $OUT_DIR/bindings.rs file.
    // let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    // bindings
    //     .write_to_file(out_path.join("bindings.rs"))
    //     .expect("Couldn't write bindings!");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    #[cfg(feature = "buildtime_bindgen")]
    {
        bindings::write_to_out_dir("wrapper.h", &out_path);
    }
    #[cfg(not(feature = "buildtime_bindgen"))]
    {
        use std::fs;
        fs::copy("bindings/duktape_binding.rs", out_path)
                .expect("Could not copy bindings to output directory");
    }

    let mut builder = cc::Build::new();

    builder
        .file("duktape-2.3.0/src/duktape.c")
        .flag_if_supported("-fomit-frame-pointer")
        .flag_if_supported("-fstrict-aliasing");
    // .flag_if_supported("-fprofile-generate")
    let profile = env::var("PROFILE").unwrap();
    if profile == "release" {
        builder.opt_level(2);
    }

    builder.compile("libduktape.a");
}



#[cfg(feature = "buildtime_bindgen")]
mod bindings {

    extern crate bindgen;

    use std::path::{Path};
    use std::fs;


    pub fn write_to_out_dir(header: &str, out_path: &Path) {
        let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(header)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    // let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");

    fs::create_dir("bindings").unwrap_or(());
    fs::copy(out_path, "bindings/duktape_binding.rs").expect("could not copy bindings");

    }
}