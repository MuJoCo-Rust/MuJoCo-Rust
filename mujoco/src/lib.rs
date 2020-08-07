//! Provides safe bindings to [MuJoCo](http://www.mujoco.org/index.html), a physics
//! simulator commonly used for robotics and machine learning.

pub mod model;
mod vfs;

use lazy_static::lazy_static;
use std::ffi::{CStr, CString};

pub(crate) mod helpers;

lazy_static! {
    /// The location of the MuJoCo key
    ///
    /// By default this is ~/.mujoco/mjkey.txt, but can be overriden via the
    /// `MUJOCO_RS_KEY_LOC` environment variable
    pub static ref KEY_LOC: String = match std::env::var("MUJOCO_RS_KEY_LOC") {
        Ok(loc) => loc,
        Err(std::env::VarError::NotPresent) => dirs::home_dir()
            .expect(
                "Could not find home directory when attempting to use default mujoco key \
                location. Consider setting `MUJOCO_RS_KEY_LOC`."
            )
            .join(".mujoco").join("mjkey.txt").to_str().unwrap().to_owned(),
        Err(std::env::VarError::NotUnicode(_)) => panic!("`MUJOCO_RS_KEY_LOC` must be unicode!")
    };
}

/// Activates MuJoCo using the default key [`KEY_LOC`]
///
/// [`KEY_LOC`]: struct.KEY_LOC.html
pub fn activate() {
    let s: &str = &KEY_LOC;
    activate_from_str(s)
}

/// Deactivates MuJoCo
///
/// Note that this globally deactivates MuJoCo, so make sure sure that other
/// code doesn't expect it to be activated when this is called
pub fn deactivate() {
    unsafe { mujoco_sys::mj_deactivate() }
}

/// Activates MuJoCo from a the key's filepath
///
/// # Panics
/// Panics if there is an error getting the mujoco key
pub fn activate_from_str(key_loc: impl AsRef<str>) {
    let key_loc = CString::new(key_loc.as_ref()).unwrap();
    activate_from_cstr(key_loc)
}

/// Activates MuJoCo from a the key's filepath as a c-style string
///
/// # Panics
/// Panics if there is an error getting the mujoco key
pub fn activate_from_cstr(key_loc: impl AsRef<CStr>) {
    let key_loc = key_loc.as_ref();
    let activate_result;
    unsafe {
        activate_result = mujoco_sys::mj_activate(key_loc.as_ptr());
    }
    if activate_result != 1 {
        unreachable!("If activation fails, mujoco calls error handler and terminates.")
    }
}

#[cfg(test)]
mod tests {

    use std::ffi::CString;
    #[test]
    fn activate() {
        let s: &str = &super::KEY_LOC;

        super::activate();
        super::activate_from_str(s);
        super::activate_from_cstr(CString::new(s).unwrap());
    }
}
