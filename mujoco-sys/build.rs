extern crate bindgen;

use bindgen::EnumVariation;
use std::env;
use std::fs::read_dir;
use std::path::PathBuf;

fn main() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    let mj_path = dirs::home_dir()
        .expect("Could not locate home directory!")
        .join(".mujoco")
        .join("mujoco200");
    let mj_bin = mj_path.join("bin");
    let mj_include = mj_path.join("include");

    println!("cargo:rustc-link-search={}", mj_bin.to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib=mujoco200");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");
    for p in read_dir(mj_bin)
        .unwrap()
        .chain(read_dir(&mj_include).unwrap())
    {
        let p = p.unwrap().path();
        println!("cargo:rerun-if-changed={}", p.to_str().unwrap());
    }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg(format!("-I{}", &mj_include.to_str().unwrap()))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))

        .whitelist_type(r"(?i)mj.*")
        .whitelist_function(r"(?i)mj.*")
        .whitelist_var(r"(?i)mj.*")

        // Use strong newtypes for C enums
        .default_enum_style(EnumVariation::NewType{is_bitfield: false})
        .size_t_is_usize(true)

        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
