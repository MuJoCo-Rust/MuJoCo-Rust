use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/generator.rs"));

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    generate();

    let (prefix, dyl_ext, default_install) = match env::var("CARGO_CFG_UNIX") {
        Ok(_) => (
            "lib",
            if env::var("CARGO_CFG_TARGET_VENDOR").unwrap() == "apple" {
                "dylib"
            } else {
                "so"
            },
            "/usr/local",
        ),
        _ => match env::var("CARGO_CFG_WINDOWS") {
            Ok(_) => ("", "dll", "C:\\Program Files\\MuJoCo"),
            _ => ("", "", ""),
        },
    };

    let lib_file = format!("{}mujoco.{}", prefix, dyl_ext);

    // Avoid linking to mujoco in docs.rs as it won't exist in that environment
    if option_env!("DOCS_RS").is_none() {
        let mj_root = match (env::var("MUJOCO_DIR"), env::var("MUJOCO_PREFIX")) {
            (Ok(dir), _) | (Err(..), Ok(dir)) => dir,
            (Err(..), Err(..)) => default_install.to_string(),
        };
        let mj_root = PathBuf::from_str(&mj_root).expect("Unable to get path");

        let mj_lib = match env::var("CARGO_CFG_WINDOWS") {
            Ok(_) => mj_root.join("bin"),
            _ => mj_root.join("lib"),
        };

        let path = mj_lib.join(&lib_file);

        // Compile-time link location
        println!("cargo:rustc-link-search={}", mj_lib.to_str().unwrap());
        println!("cargo:rustc-link-lib=dylib=mujoco");

        match env::var("CARGO_CFG_WINDOWS") {
            Ok(_) => {
                std::fs::read(path.to_str().unwrap())
                    .expect(format!("Expected file at {}", &lib_file).as_str());

                println!(
                    "cargo:rerun-if-changed={}",
                    Path::new(path.to_str().unwrap()).to_str().unwrap()
                );
            }
            _ => {
                println!(
                    "cargo:rerun-if-changed={}",
                    std::fs::read_link(path.to_str().unwrap())
                        .expect(
                            format!("Expected symbolic link to {}", &lib_file).as_str()
                        )
                        .to_str()
                        .unwrap()
                );
            }
        }
    }
}
