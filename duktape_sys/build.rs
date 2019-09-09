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
        bindings::write_to_out_dir("bindings/duktape.h", &out_path);
    }
    #[cfg(not(feature = "buildtime_bindgen"))]
    {
        use std::fs;
        fs::copy("bindings/duktape_binding.rs", out_path)
                .expect("Could not copy bindings to output directory");
    }

    let mut builder = cc::Build::new();

    builder
        .file("bindings/duktape.c")
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

    use std::path::{Path, PathBuf};
    use std::env;
    use std::fs;



    pub fn write_to_out_dir(header: &str, out_path: &Path) {

        let output_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("duktape");
        fs::remove_dir_all(&output_dir).unwrap_or(());
        fs::create_dir_all(&output_dir).expect("Unable to create output directory");

        use std::process;
        let o = process::Command::new("python2")
        .args(&["tools/configure.py", "--output-directory", output_dir.to_str().unwrap(), 
            "-DDUK_USE_SYMBOL_BUILTIN",
            "-UDUK_USE_FILE_IO",
            //"--dll",
            "-DDUK_USE_FASTINT"
            ])
        .current_dir("duktape-2.3.0")
        .output().expect("Unable to configure duktape build scripts");

        assert!(o.status.success());

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
        fs::copy(output_dir.join("duk_config.h"), "bindings/duk_config.h").expect("could not copy duk_config.h");
        fs::copy(output_dir.join("duktape.h"), "bindings/duktape.h").expect("could not copy duktape.h");
        fs::copy(output_dir.join("duktape.c"), "bindings/duktape.c").expect("could not copy duktape.c");


    }
}