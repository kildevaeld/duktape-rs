
use std::fs;
use std::env;
use std::path::PathBuf;

fn main() {

    #[cfg(feature = "buildtime_bindgen")]
    {
        use std::process;
        let o = process::Command::new("node")
        .args(&["node_modules/.bin/gulp", "default"])
        .current_dir("userland")
        .output().unwrap();

        assert!(o.status.success());
    }

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());


    for entry in fs::read_dir("userland/dist").unwrap() {
           
            let path = entry.unwrap().path();
            let ext = path.extension().unwrap();
            let name = path.file_name().unwrap();
            if ext == "js" {
                fs::copy(&path, out_path.join(name)).unwrap();
            }
    }

}