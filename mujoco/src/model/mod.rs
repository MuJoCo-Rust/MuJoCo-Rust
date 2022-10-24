use std::ffi::CString;

use crate::VFS;

pub use crate::re_exports::ObjType;
use mujoco_rs_sys::no_render::mjModel;
use std::ffi::CStr;

type Id = u16;

/// A MuJoCo model
#[derive(Debug)]
pub struct Model {
    pub(crate) ptr: *mut mjModel,
}
// Creation, serialization, and deserialization funcs
impl Model {
    /// Loads a `Model` from a path to an XML file
    ///
    /// # Panics
    /// Panics if the xml is invalid or the file doesn't exist
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
        err_buf.resize(1024, b'\0'); // Allocate and initialize 1024 null bytes

        let model_ptr = unsafe {
            mujoco_rs_sys::no_render::mj_loadXML(
                filepath.as_ptr(),
                std::ptr::null(),
                err_buf.as_mut_ptr() as *mut std::os::raw::c_char,
                err_buf.len() as std::os::raw::c_int,
            )
        };
        from_xml_helper(model_ptr, err_buf)
    }

    /// Loads a `Model` from an XML string
    ///
    /// # Panics
    /// Panics if the xml is invalid
    pub fn from_xml_str(xml: impl AsRef<str>) -> Result<Self, String> {
        let xml = xml.as_ref();
        let filename = "from_xml_str";
        let filename_cstr = CString::new(filename).unwrap();
        VFS.with(|rcell| {
            let mut vfs = rcell.borrow_mut();
            vfs.add_file(filename, xml.as_bytes()).unwrap();

            let mut err_buf = Vec::new();
            // TODO: Would it be safe to just allocate w/o init?
            err_buf.resize(1024, b'\0'); // Allocate and initialize 1024 null bytes

            let model_ptr = unsafe {
                mujoco_rs_sys::no_render::mj_loadXML(
                    filename_cstr.as_ptr(),
                    &*vfs.vfs,
                    err_buf.as_mut_ptr() as *mut std::os::raw::c_char,
                    err_buf.len() as std::os::raw::c_int,
                )
            };
            vfs.delete_file(filename);
            from_xml_helper(model_ptr, err_buf)
        })
    }

    /// Loads a model from a slice of bytes, interpreted as the MJB format
    ///
    /// # Panics
    /// Panics if the bytes are an invalid model
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        let filename = "from_bytes";
        let filename_cstr = CString::new(filename).unwrap();
        VFS.with(|rcell| {
            let mut vfs = rcell.borrow_mut();

            // Allocate with null bytes
            // (&*vfs.vfs).resize(1024, b'\0');

            vfs.add_file(filename, bytes).unwrap();

            let model_ptr = unsafe {
                mujoco_rs_sys::no_render::mj_loadModel(
                    filename_cstr.as_ptr(),
                    &*vfs.vfs,
                )
            };
            vfs.delete_file(filename);
            Self { ptr: model_ptr }
        })
    }

    /// Serializes the `Model` into a binary vector
    pub fn to_vec(&self) -> Vec<u8> {
        let nbytes = unsafe { mujoco_rs_sys::no_render::mj_sizeModel(self.ptr) };
        let mut buf: Vec<u8> = Vec::with_capacity(nbytes as usize);
        buf.resize(nbytes as usize, 0);
        unsafe {
            mujoco_rs_sys::no_render::mj_saveModel(
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
// Accessors
impl Model {
    /// Gets the low level [`mjModel`] that the `Model` uses under the hood
    pub fn ptr(&self) -> *mut mjModel {
        self.ptr
    }

    /// Converts `name` to an id that serves as an offset into the arrays in the
    /// underlying [`mjModel`]
    pub fn name_to_id(&self, obj_type: ObjType, name: &str) -> Option<Id> {
        self.cstr_name_to_id(obj_type, &CString::new(name).unwrap())
    }

    pub fn cstr_name_to_id(&self, obj_type: ObjType, name: &CStr) -> Option<Id> {
        let result = unsafe {
            mujoco_rs_sys::no_render::mj_name2id(
                self.ptr,
                obj_type as std::os::raw::c_int,
                name.as_ptr(),
            )
        };
        if result == -1 {
            None
        } else {
            Some(result as Id)
        }
    }

    pub fn id_to_name(&self, obj_type: ObjType, id: Id) -> Option<&str> {
        let cstr = unsafe {
            mujoco_rs_sys::no_render::mj_id2name(
                self.ptr,
                obj_type as std::os::raw::c_int,
                id as std::os::raw::c_int,
            )
        };
        if cstr == std::ptr::null() {
            return None;
        }
        let cstr = unsafe { CStr::from_ptr(cstr) };
        Some(cstr.to_str().expect("Expected valid unicode for the name!"))
    }
}
impl Drop for Model {
    fn drop(&mut self) {
        unsafe { mujoco_rs_sys::no_render::mj_deleteModel(self.ptr) };
    }
}
impl Clone for Model {
    fn clone(&self) -> Self {
        let ptr = unsafe {
            mujoco_rs_sys::no_render::mj_copyModel(std::ptr::null_mut(), self.ptr)
        };
        Self { ptr }
    }
}

/// Helper function for loading a model from xml
fn from_xml_helper(model_ptr: *mut mjModel, err_buf: Vec<u8>) -> Result<Model, String> {
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
    use super::*;
    use crate::activate;
    use crate::tests::{SIMPLE_XML, SIMPLE_XML_PATH};

    fn check_expected_ids(m: &Model) {
        assert_eq!(m.name_to_id(ObjType::BODY, "world").unwrap(), 0);
        assert_eq!(m.name_to_id(ObjType::LIGHT, "light0").unwrap(), 0);
        assert_eq!(m.name_to_id(ObjType::GEOM, "geom0").unwrap(), 0);
        assert_eq!(m.name_to_id(ObjType::BODY, "body1").unwrap(), 1);
        assert_eq!(m.name_to_id(ObjType::JOINT, "joint0").unwrap(), 0);
        assert_eq!(m.name_to_id(ObjType::GEOM, "geom1").unwrap(), 1);
        assert_eq!(unsafe { *(*m.ptr()).geom_size.offset(3) }, 0.1);
    }

    #[test]
    fn from_xml() {
        activate();
        let m = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();

        // Check expected values
        check_expected_ids(&m);
    }

    #[test]
    fn from_xml_str() {
        activate();
        let model_xml = Model::from_xml_str(*SIMPLE_XML).unwrap();
        // let model_file = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
        // assert_eq!(model_xml.to_vec(), model_file.to_vec());
        check_expected_ids(&model_xml);
    }

    #[test]
    fn from_bytes() {
        activate();
        let model_xml = Model::from_xml/*_str*/(&*SIMPLE_XML_PATH).unwrap();
        let model_xml_bytes = model_xml.to_vec();
        let model_from_bytes = Model::from_bytes(&model_xml_bytes);
        assert_eq!(model_from_bytes.to_vec(), model_xml_bytes);
    }

    #[test]
    fn serialize() {
        activate();
        let m = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
        let serialized = m.to_vec();
        assert_eq!(serialized.len(), unsafe {
            mujoco_rs_sys::no_render::mj_sizeModel(m.ptr)
        } as usize);
        println!("Serialized data: {:?}", serialized);
    }

    #[test]
    fn name_to_id() {
        activate();
        let m = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();

        // Check expected values
        check_expected_ids(&m);

        // Check that non-existent names give `None`
        assert_eq!(m.name_to_id(ObjType::BODY, "asdf"), None);
        assert_eq!(m.name_to_id(ObjType::BODY, "body0"), None);
        assert_eq!(m.name_to_id(ObjType::GEOM, "geom2"), None);
        assert_eq!(m.name_to_id(ObjType::CAMERA, "cam"), None);
    }

    #[test]
    fn id_to_name() {
        activate();
        let m = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();

        // Check expected values
        assert_eq!(m.id_to_name(ObjType::BODY, 0).unwrap(), "world");
        assert_eq!(m.id_to_name(ObjType::LIGHT, 0).unwrap(), "light0");
        assert_eq!(m.id_to_name(ObjType::GEOM, 0).unwrap(), "geom0");
        assert_eq!(m.id_to_name(ObjType::BODY, 1).unwrap(), "body1");
        assert_eq!(m.id_to_name(ObjType::JOINT, 0).unwrap(), "joint0");
        assert_eq!(m.id_to_name(ObjType::GEOM, 1).unwrap(), "geom1");

        // Check that non-existent ids give `None`
        assert_eq!(m.id_to_name(ObjType::BODY, 2), None);
        assert_eq!(m.id_to_name(ObjType::LIGHT, 1), None);
        assert_eq!(m.id_to_name(ObjType::GEOM, 2), None);
        assert_eq!(m.id_to_name(ObjType::CAMERA, 0), None);
    }
}
