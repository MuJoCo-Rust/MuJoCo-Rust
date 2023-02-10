//! Provides safe bindings to [MuJoCo](http://www.mujoco.org/index.html), a physics
//! simulator commonly used for robotics and machine learning.

pub mod body;
pub mod geom;
pub mod mesh;
pub mod model;
mod re_exports;
pub mod sim;
pub mod state;
mod vfs;

pub use body::Body;
pub use geom::Geom;
pub use mesh::Mesh;
pub use model::Model;
pub use re_exports::GeomType;
pub use sim::Simulation;
pub use state::State;

use std::cell::RefCell;

pub(crate) mod helpers;

// Using a thread-local VFS is how we enable the use of temporary VFS objects in
// a global storage without accidentally getting filename collisions or thread
// safety issues. The RefCell is there to allow us to mutate safely (it could
// probably be mutated via unsafe code everywhere but that is a minor speed
// improvement for many unsafe LOC)
thread_local! {
    static VFS: RefCell<vfs::Vfs> = RefCell::new(vfs::Vfs::new());
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    lazy_static! {
        pub(crate) static ref PKG_ROOT: std::path::PathBuf =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .canonicalize()
                .expect("Could not resolve absolute path for package root!");
        pub(crate) static ref SIMPLE_XML_PATH: std::path::PathBuf =
            PKG_ROOT.join("tests").join("res").join("simple.xml");
        pub(crate) static ref SIMPLE_XML: &'static str = r#"<mujoco>
    <worldbody>
        <light name="light0" diffuse=".5 .5 .5" pos="0 0 3" dir="0 0 -1"/>
        <geom name="geom0" type="plane" size="1 1 0.1" rgba=".9 0 0 1"/>
        <body name="body1" pos="0 0 1">
            <joint name="joint0" type="free"/>
            <geom name="geom1" type="box" size=".1 .2 .3" rgba="0 .9 0 1"/>
        </body>
    </worldbody>
</mujoco>"#;
    }
}
