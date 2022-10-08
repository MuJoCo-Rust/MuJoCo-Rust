#!/usr/bin/env run-cargo-script
//! Used by cargo-script...
//!
//! ```cargo
//! [dependencies]
//! bindgen = "*"
//! dirs = "*"
//! ```

extern crate bindgen;

use bindgen::callbacks::{EnumVariantValue, ParseCallbacks};
use bindgen::EnumVariation;
use std::env;
use std::fs::read_dir;
use std::path::PathBuf;

#[derive(Debug)]
struct EnumPrefixStripper {}
impl ParseCallbacks for EnumPrefixStripper {
    fn enum_variant_name(
        &self,
        _enum_name: Option<&str>,
        _original_variant_name: &str,
        _variant_value: EnumVariantValue,
    ) -> Option<String> {
        let enum_name = _enum_name?;
        if !enum_name.starts_with("enum ") {
            return None;
        }
        let enum_name = enum_name.strip_prefix("enum ").unwrap();
        let original = _original_variant_name;
        {
            let edge_case = match original {
                "mjTEXTURE_2D" => Some("TWO_D"),

                "mjFONTSCALE_50" => Some("SCALE_50"),
                "mjFONTSCALE_100" => Some("SCALE_100"),
                "mjFONTSCALE_150" => Some("SCALE_150"),
                "mjFONTSCALE_200" => Some("SCALE_200"),
                "mjFONTSCALE_250" => Some("SCALE_250"),
                "mjFONTSCALE_300" => Some("SCALE_300"),

                _ => None,
            };
            if let Some(s) = edge_case {
                println!("Found edge case! {}::{} => {0}::{}", enum_name, original, s);
                return Some(s.to_owned());
            }
        }

        let first_underscore = original.find("_")?;
        let suffix = &original[first_underscore + 1..];
        if suffix.starts_with(char::is_numeric) {
            panic!(
                "Encountered {}::{} which is not a valid identifier! Add an edge case.",
                enum_name, original
            );
        }
        println!("{}::{} => {0}::{}", enum_name, original, suffix);
        Some(suffix.to_owned())
    }
}

fn main() {
    let mj_path = dirs::home_dir()
        .expect("Could not locate home directory!")
        .join(".local")
        .join("mujoco");
    let mj_include = mj_path.join("include");

    let builder_helper = |b: bindgen::Builder, whitelist: &str| -> bindgen::Builder {
        b.header_contents("wrapper.h", r#"#include "mujoco/mujoco.h""#)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .whitelist_type(whitelist)
            .whitelist_function(whitelist)
            .whitelist_var(whitelist)
            .rustified_enum(r"_?mjt.+")
            .bitfield_enum(r"_?mjt.+Bit")
            .default_enum_style(EnumVariation::NewType { is_bitfield: false })
            .parse_callbacks(Box::new(EnumPrefixStripper{}))
            // MuJoCo mjtWhatevers enums are not actually used in the API, so this will
            // make re-exposing for the user API easier
            .size_t_is_usize(true)
            .derive_default(true)
            .clang_arg("-I".to_owned() + mj_include.to_str().unwrap())
    };

    // Whitelist all mj* except mjr*
    let no_render_binds = builder_helper(bindgen::Builder::default(), r"(?i)mj[^r].*")
        .opaque_type("mjVFS")
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
