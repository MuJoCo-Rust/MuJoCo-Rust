extern crate bindgen;

use bindgen::EnumVariation;
use std::env;
use std::fs::read_dir;
use std::path::PathBuf;

fn main() {
    // Avoid touching mj_path when in docs.rs, as it won't exist in that environment
    if option_env!("DOCS_RS").is_some() {
        let mj_path = dirs::home_dir()
            .expect("Could not locate home directory!")
            .join(".mujoco")
            .join("mujoco200");
        let mj_bin = mj_path.join("bin");

        println!("cargo:rustc-link-search={}", mj_bin.to_str().unwrap());
        println!("cargo:rustc-link-lib=dylib=mujoco200");

        for p in read_dir(mj_bin).unwrap() {
            let p = p.unwrap().path();
            println!("cargo:rerun-if-changed={}", p.to_str().unwrap());
        }
    }

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the headers
        // included in `wrapper.h` or their transitive includes change.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .whitelist_type(r"(?i)mj.*")
        .whitelist_function(r"(?i)mj.*")
        .whitelist_var(r"(?i)mj.*")
        .default_enum_style(EnumVariation::NewType { is_bitfield: false })
        .size_t_is_usize(true)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
