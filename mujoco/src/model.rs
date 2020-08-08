use crate::vfs::Vfs;
use std::ffi::CString;

use crate::VFS;

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
        from_xml_helper(model_ptr, err_buf)
    }

    pub fn from_xml_str(xml: impl AsRef<str>) -> Result<Self, String> {
        let xml = xml.as_ref();
        let filename = "from_xml_str";
        let filename_cstr = CString::new(filename).unwrap();
        VFS.with(|rcell| {
            let mut vfs = rcell.borrow_mut();
            vfs.add_file(filename, xml.as_bytes()).unwrap();

            let mut err_buf = Vec::new();
            // TODO: Would it be safe to just allocate w/o init?
            err_buf.resize(1000, b'\0'); // Allocate and initialize 1000 null bytes

            let model_ptr = unsafe {
                mujoco_sys::no_render::mj_loadXML(
                    filename_cstr.as_ptr(),
                    &vfs.vfs,
                    err_buf.as_mut_ptr() as *mut std::os::raw::c_char,
                    err_buf.len() as std::os::raw::c_int,
                )
            };
            vfs.delete_file(filename);
            from_xml_helper(model_ptr, err_buf)
        })
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

/// Helper function for loading a model from xml
fn from_xml_helper(
    model_ptr: *mut mujoco_sys::no_render::_mjModel,
    err_buf: Vec<u8>,
) -> Result<Model, String> {
    debug_assert_ne!(model_ptr, std::ptr::null_mut());
    debug_assert_ne!(err_buf.len(), 0);
    let err_str = crate::helpers::convert_err_buf(err_buf);

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
        static ref SIMPLE_XML: &'static str = r#"
            <mujoco>
                <worldbody>
                    <light diffuse=".5 .5 .5" pos="0 0 3" dir="0 0 -1"/>
                    <geom type="plane" size="1 1 0.1" rgba=".9 0 0 1"/>
                    <body pos="0 0 1">
                        <joint type="free"/>
                        <geom type="box" size=".1 .2 .3" rgba="0 .9 0 1"/>
                    </body>
                </worldbody>
            </mujoco>"#;
    }

    #[test]
    fn from_xml() {
        activate();
        Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
    }

    #[test]
    fn from_xml_str() {
        activate();
        let model_xml = Model::from_xml_str(*SIMPLE_XML).unwrap();
        let model_file = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
        assert_eq!(model_xml.to_vec(), model_file.to_vec());
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
