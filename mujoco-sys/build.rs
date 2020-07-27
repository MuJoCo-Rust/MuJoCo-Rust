extern crate bindgen;

use bindgen::EnumVariation;
use std::env;
use std::fs::read_dir;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    // Avoid linking to mujoco in docs.rs as it won't exist in that environment
    if option_env!("DOCS_RS").is_none() {
        let mj_path = dirs::home_dir()
            .expect("Could not locate home directory!")
            .join(".mujoco")
            .join("mujoco200");
        let mj_bin = mj_path.join("bin");

        println!("cargo:rustc-link-search={}", mj_bin.to_str().unwrap());
        if cfg!(feature = "mj-render") {
            println!("cargo:rustc-link-lib=dylib=mujoco200");
        } else {
            println!("cargo:rustc-link-lib=dylib=mujoco200nogl");
        }

        for p in read_dir(mj_bin).unwrap() {
            let p = p.unwrap().path();
            println!("cargo:rerun-if-changed={}", p.to_str().unwrap());
        }
    }

    fn builder_helper(b: bindgen::Builder, whitelist: &str) -> bindgen::Builder {
        b.header("wrapper.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .whitelist_type(whitelist)
            .whitelist_function(whitelist)
            .whitelist_var(whitelist)
            .default_enum_style(EnumVariation::NewType { is_bitfield: false })
            .size_t_is_usize(true)
    }

    // Whitelist all mj* except mjr*
    let no_render_binds = builder_helper(bindgen::Builder::default(), r"(?i)mj[^r].*")
        .generate()
        .expect("Unable to generate bindings");
    // Whitelist only mjr*. Need to also include _mjr* due to non-recursive
    let render_binds = builder_helper(bindgen::Builder::default(), r"_?mjr.*")
        .whitelist_recursively(false)
        .raw_line("pub use crate::no_render::*;")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/whatever.rs files.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    no_render_binds
        .write_to_file(out_path.join("no_render.rs"))
        .expect("Couldn't write bindings!");
    render_binds
        .write_to_file(out_path.join("render.rs"))
        .expect("Couldn't write bindings!");
}
