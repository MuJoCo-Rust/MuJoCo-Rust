use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/generator.rs"));

fn get_output_path() -> PathBuf {
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string)
        .join("..")
        .join("target")
        .join(build_type);
    path
}

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    generate();

    let default_install_path = dirs::home_dir()
        .expect("Could not locate home directory!")
        .join(".local")
        .join("mujoco");
    let default_install_path = default_install_path.to_str().unwrap();

    let (prefix, dyl_ext, default_install) = match env::var("CARGO_CFG_UNIX") {
        Ok(_) => (
            "lib",
            if env::var("CARGO_CFG_TARGET_VENDOR").unwrap() == "apple" {
                "dylib"
            } else {
                "so"
            },
            default_install_path,
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

        let mj_lib_windows = mj_root.join("bin");
        let mj_lib_posix = mj_root.join("bin");

        let path = match env::var("CARGO_CFG_WINDOWS") {
            Ok(_) => mj_lib_windows.join(&lib_file),
            _ => mj_lib_posix.join(&lib_file),
        };

        println!("cargo:rustc-link-lib=dylib=mujoco");
        println!("cargo:rustc-link-search={}", mj_lib_posix.to_str().unwrap());
        println!("cargo:rustc-link-lib=GL");
        println!("cargo:rustc-link-lib=GLEW");

        // Copy mujoco.dll to target directory on Windows targets
        if env::var("CARGO_CFG_WINDOWS").is_ok() {
            let target_dir = get_output_path();
            let src = Path::join(
                &env::current_dir().unwrap(),
                mj_lib_windows.join("mujoco.dll"),
            );

            fs::create_dir_all(&target_dir).unwrap();
            let dest = Path::join(Path::new(&target_dir), Path::new("mujoco.dll"));
            std::fs::copy(src, dest).unwrap();
        }

        std::fs::read(path.to_str().unwrap())
            .unwrap_or_else(|_| panic!("Expected file at {}", &lib_file));

        println!(
            "cargo:rerun-if-changed={}",
            Path::new(path.to_str().unwrap()).to_str().unwrap()
        );
    }
}
