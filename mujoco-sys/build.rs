use std::fs::read_dir;

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
}
