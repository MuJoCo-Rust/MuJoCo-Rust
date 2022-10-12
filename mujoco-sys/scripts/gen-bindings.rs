#!/usr/bin/env run-cargo-script
//! Used by cargo-script...
//!
//! ```cargo
//! [dependencies]
//! bindgen = "*"
//! dirs = "*"
//! ```
use std::env;
use std::fs::read_dir;
use std::path::PathBuf;

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/scripts/generator.rs"
));

fn main()
{
    generate();
}
