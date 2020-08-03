use std::ffi::CString;

/// A MuJoCo model
pub struct Model {
    ptr: *mut mujoco_sys::no_render::mjModel,
}
impl Model {
    /// Loads a `Model` from a path to an XML file.
    ///
    /// # Panics
    /// Panics if `path` is not valid Unicode or if it has a null byte in its
    /// interior.
    pub fn from_xml(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err("File doesn't exist!".to_owned());
        }
        let filepath =
            CString::new(path.to_str().expect("Could not convert `path` to unicode!"))
                .expect("`path` had an unexpected null byte in its interior!");

        const ERR_STR_CAPACITY: usize = 1000;
        let err_str = CString::new(Vec::with_capacity(ERR_STR_CAPACITY))
            .unwrap()
            .into_raw();

        let model_ptr = unsafe {
            mujoco_sys::no_render::mj_loadXML(
                filepath.as_ptr(),
                std::ptr::null(),
                err_str,
                ERR_STR_CAPACITY as std::os::raw::c_int,
            )
        };

        // TODO: Investigate what happens if the string buffer is totally filled up.
        // Will the buffer still be null terminated? If not, this can cause a
        // segfault.
        let err_str = unsafe { CString::from_raw(err_str) };
        let err_str = err_str.into_string().expect(
            "Encountered an error from `mj_loadXML()` that was not valid UTF-8!",
        );
        debug_assert_eq!(err_str.capacity(), ERR_STR_CAPACITY);

        if !err_str.is_empty() {
            return Err(err_str);
        }
        if model_ptr == std::ptr::null_mut() {
            unreachable!(
                "It shouldn't be possible to get a null pointer from mujoco \
                    without an error message!"
            );
        }
        Ok(Model { ptr: model_ptr })
    }
}
impl Drop for Model {
    fn drop(&mut self) {
        unsafe { mujoco_sys::no_render::mj_deleteModel(self.ptr) };
    }
}
