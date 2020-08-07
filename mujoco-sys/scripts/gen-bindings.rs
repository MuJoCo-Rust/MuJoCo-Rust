#!/usr/bin/env run-cargo-script
//! Used by cargo-script...
//!
//! ```cargo
//! [dependencies]
//! bindgen = "0.54.1"
//! dirs = "3.0"
//! ```

extern crate bindgen;

use bindgen::EnumVariation;
use std::env;
use std::fs::read_dir;
use std::path::PathBuf;

fn main() {
    let mj_path = dirs::home_dir()
        .expect("Could not locate home directory!")
        .join(".mujoco")
        .join("mujoco200");
    let mj_include = mj_path.join("include");

    let builder_helper = |b: bindgen::Builder, whitelist: &str| -> bindgen::Builder {
        b.header_contents("wrapper.h", r#"#include "mujoco.h""#)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .whitelist_type(whitelist)
            .whitelist_function(whitelist)
            .whitelist_var(whitelist)
            .default_enum_style(EnumVariation::NewType { is_bitfield: false })
            .size_t_is_usize(true)
            .derive_default(true)
            .clang_arg("-I".to_owned() + mj_include.to_str().unwrap())
    };

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

    // Write the bindings to the `generated` directory.
    let out_path = PathBuf::from("generated");
    no_render_binds
        .write_to_file(out_path.join("no_render.rs"))
        .expect("Couldn't write bindings!");
    render_binds
        .write_to_file(out_path.join("render.rs"))
        .expect("Couldn't write bindings!");
}
