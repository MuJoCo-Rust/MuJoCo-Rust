use std::fs::read_dir;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    // Avoid linking to mujoco in docs.rs as it won't exist in that environment
    if option_env!("DOCS_RS").is_none() {
        let mj_path = dirs::home_dir()
            .expect("Could not locate home directory!")
            .join(".local")
            .join("mujoco");
        let mj_bin = mj_path.join("lib");

        // Compile-time link location
        println!("cargo:rustc-link-search={}", mj_bin.to_str().unwrap());
        println!("cargo:rustc-link-lib=dylib=mujoco");

        for p in read_dir(mj_bin).unwrap() {
            let p = p.unwrap().path();
            println!("cargo:rerun-if-changed={}", p.to_str().unwrap());
        }
    }
}
