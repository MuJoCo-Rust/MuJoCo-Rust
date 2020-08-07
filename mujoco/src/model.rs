use std::ffi::CString;

/// A MuJoCo model
#[derive(Debug)]
pub struct Model {
    ptr: *mut mujoco_sys::no_render::mjModel,
}
impl Model {
    /// Loads a `Model` from a path to an XML file
    ///
    /// # Panics
    /// Panics if `path` is not valid Unicode or if it has a null byte in its
    /// interior
    pub fn from_xml(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err("File doesn't exist!".to_owned());
        }
        let filepath =
            CString::new(path.to_str().expect("Could not convert `path` to unicode!"))
                .expect("`path` had an unexpected null byte in its interior!");

        let mut err_buf = Vec::new();
        // TODO: Would it be safe to just allocate w/o init?
        err_buf.resize(1000, b'\0'); // Allocate and initialize 1000 null bytes

        let model_ptr = unsafe {
            mujoco_sys::no_render::mj_loadXML(
                filepath.as_ptr(),
                std::ptr::null(),
                err_buf.as_mut_ptr() as *mut std::os::raw::c_char,
                err_buf.len() as std::os::raw::c_int,
            )
        };

        let err_str = CString::new(err_buf).unwrap_or_else(|e| {
            let nul_pos = e.nul_position();
            let mut err_buf = e.into_vec();
            debug_assert!(nul_pos < err_buf.len());
            // Shrinks to all chars up to but not including nul byte
            err_buf.resize_with(nul_pos, Default::default);
            debug_assert_eq!(nul_pos, err_buf.len());
            debug_assert!(!err_buf.contains(&b'\0'));
            // This is unsafe for performance reasons, but could be switched back to a
            // safe alternative
            // Will shrink vec to fit. Not ideal.
            unsafe { CString::from_vec_unchecked(err_buf) }
        });
        let err_str = err_str.into_string().expect("`CString` was not UTF-8!");

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

    /// Serializes the `Model` into a binary vector
    pub fn to_vec(&self) -> Vec<u8> {
        let nbytes = unsafe { mujoco_sys::no_render::mj_sizeModel(self.ptr) };
        let mut buf: Vec<u8> = Vec::with_capacity(nbytes as usize);
        unsafe {
            mujoco_sys::no_render::mj_saveModel(
                self.ptr,
                std::ptr::null(),
                buf.as_mut_ptr() as *mut std::os::raw::c_void,
                nbytes,
            );
            buf.set_len(nbytes as usize);
        };
        buf
    }
}
impl Drop for Model {
    fn drop(&mut self) {
        unsafe { mujoco_sys::no_render::mj_deleteModel(self.ptr) };
    }
}
impl Clone for Model {
    fn clone(&self) -> Self {
        let ptr = unsafe {
            mujoco_sys::no_render::mj_copyModel(std::ptr::null_mut(), self.ptr)
        };
        Self { ptr }
    }
}

#[cfg(test)]
mod tests {
    use super::Model;
    use crate::activate;

    use lazy_static::lazy_static;

    lazy_static! {
        static ref PKG_ROOT: std::path::PathBuf =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .canonicalize()
                .expect("Could not resolve absolute path for package root!");
        static ref SIMPLE_XML_PATH: std::path::PathBuf =
            PKG_ROOT.join("tests").join("res").join("simple.xml");
    }

    #[test]
    fn should_work() {
        assert!(true)
    }

    #[test]
    fn from_xml() {
        activate();
        Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
    }

    #[test]
    fn serialize() {
        activate();
        let m = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
        let serialized = m.to_vec();
        assert_eq!(serialized.len(), unsafe {
            mujoco_sys::no_render::mj_sizeModel(m.ptr)
        } as usize);
        println!("Serialized data: {:?}", serialized);
    }

    #[test]
    fn clone() {
        activate();
        let m_original = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
        let m_cloned = m_original.clone();
        assert_eq!(m_original.to_vec(), m_cloned.to_vec());
    }
}
