//! Provides safe bindings to [MuJoCo](http://www.mujoco.org/index.html), a physics
//! simulator commonly used for robotics and machine learning.

pub mod model;

use lazy_static::lazy_static;
use static_assertions::const_assert_eq;
use std::ffi::{CStr, CString};

lazy_static! {
    /// The location of the MuJoCo key. By default this is ~/.mujoco/mjkey.txt, but can
    /// be overriden via the MUJOCO_RS_KEY_LOC environment variable
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

/// Activates the MuJoCo key. Returns an [`ActivationContext`], which when
/// dropped deactivates MuJoCo. Equivalent to calling
/// [`ActivationContext::new()`] with [`KEY_LOC`]
///
/// [`KEY_LOC`]: struct.KEY_LOC.html
pub fn activate() -> ActivationContext {
    let s: &str = &KEY_LOC; // Why is this needed?
    ActivationContext::new(s)
}

// Helper type to represent empty data
#[derive(Default)]
struct Empty {}
const_assert_eq!(std::mem::size_of::<Empty>(), 0);

/// Lexically scoped activation context. [`Self::new()`] activates MuJoCo, and
/// when the context is dropped, MuJoCo is deactivated
pub struct ActivationContext(Empty);
impl ActivationContext {
    /// Creates a new `ActivationContext`. Equivalent to calling [`activate()`]
    ///
    /// # Panics
    /// Panics if there is an error getting the mujoco key
    pub fn new(key_loc: impl AsRef<str>) -> Self {
        let key_loc = CString::new(key_loc.as_ref()).unwrap();
        Self::new_from_cstr(key_loc)
    }

    /// Creates a new `ActivationContext` from a c-style string
    ///
    /// # Panics
    /// Panics if there is an error getting the mujoco key
    pub fn new_from_cstr(key_loc: impl AsRef<CStr>) -> Self {
        let key_loc = key_loc.as_ref();
        let activate_result;
        unsafe {
            activate_result = mujoco_sys::mj_activate(key_loc.as_ptr());
        }
        if activate_result != 1 {
            unreachable!(
                "If activation fails, mujoco calls error handler and terminates."
            )
        }
        Self(Empty {})
    }
}
impl Drop for ActivationContext {
    fn drop(&mut self) {
        unsafe { mujoco_sys::mj_deactivate() }
    }
}
const_assert_eq!(std::mem::size_of::<ActivationContext>(), 0);

#[cfg(test)]
mod tests {

    #[test]
    fn activate() {
        super::activate();
    }
}
