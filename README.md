# mujoco-rust

MuJoCo bindings for Rust

## Installation requirements

These bindings require that MuJoCo be installed before building. The build system assumes that MuJoCo is installed into `~/.local/mujoco` on UNIX-based systems and `C:\Program Files\MuJoCo` on Windows, but any installation directory can be used, as long as either the environment variable MUJOCO_DIR or MUJOCO_PREFIX is set to the root of the installation.

This package relies on MuJoCo headers and binaries installed in your system. Hence current versioning may change for CI but with no changes to source code of Rust bindings.

## Usage

These wrappers use `mujoco-rs-sys` to provide rust bindings to the MuJoCo C API. The `mujoco-rs-sys` crate is not intended to be used directly, but instead is used by `mujoco-rust` to provide a more idiomatic rust interface to the MuJoCo API.

### Example Usage

**Cargo.toml**

```toml
[dependencies]
mujoco-rust = { version = "0.0.7" }
```

**main.rs**

```rust
let model = mujoco_rust::Model::from_xml("simple.xml".to_string()).unwrap();
let simulation = MuJoCoSimulation::new(model);
```
