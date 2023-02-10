use crate::Body;
use crate::Geom;
use crate::Mesh;
use crate::VFS;

use crate::geom::geom_type_from;
use crate::helpers::extract_indices;
use crate::helpers::extract_mesh_attribute;
use crate::helpers::extract_vector_float;
use crate::helpers::Local;
use crate::helpers::LocalFloat;

pub use crate::re_exports::ObjType;

use mujoco_rs_sys::no_render::mjModel;
use nalgebra::Quaternion;
use nalgebra::Vector3;

use std::ffi::CStr;
use std::ffi::CString;

use arrayvec::ArrayVec;

type Id = u16;

/// A MuJoCo model
#[derive(Debug)]
pub struct Model {
    pub(crate) ptr: *mut mjModel,
}

unsafe impl Send for Model {}
unsafe impl Sync for Model {}

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
        err_buf.resize(1000, b'\0'); // Allocate and initialize 1000 null bytes

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
            err_buf.resize(1000, b'\0'); // Allocate and initialize 1000 null bytes

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
        let mut buf: Vec<u8> = vec![0; nbytes as usize];
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
        #[allow(clippy::cmp_null)]
        if cstr == std::ptr::null() {
            return None;
        }
        let cstr = unsafe { CStr::from_ptr(cstr) };
        Some(cstr.to_str().expect("Expected valid unicode for the name!"))
    }

    /// Return a list of names used in a model
    pub fn names(&self) -> Vec<String> {
        let mj_model = self;
        let mj_model = unsafe { *mj_model.ptr() };

        let mut names: Vec<String> = Vec::new();
        let mut j: isize = 0;
        for _ in 0..mj_model.nnames {
            names.push(String::new());

            loop {
                let char = unsafe { *mj_model.names.offset(j) };
                if char == 0 {
                    j += 1;
                    break;
                }
                let idx = names.len() - 1;
                if char != 0 {
                    names[idx] =
                        format!("{}{}", names[names.len() - 1], (char as u8) as char);
                }
                j += 1;
            }
        }

        names
    }

    /// Returns a number of geoms in the model
    pub fn ngeom(&self) -> usize {
        unsafe {
            let mj_model = self;
            (*mj_model.ptr()).ngeom as usize
        }
    }

    /// Returns a number of generalized coordinates
    pub fn nq(&self) -> usize {
        unsafe {
            let mj_model = self;
            (*mj_model.ptr()).nq as usize
        }
    }

    /// Returns a number of activation states
    pub fn na(&self) -> usize {
        unsafe {
            let mj_model = self;
            (*mj_model.ptr()).na as usize
        }
    }

    /// Returns a number of degrees of freedom
    pub fn nv(&self) -> usize {
        unsafe {
            let mj_model = self;
            (*mj_model.ptr()).nv as usize
        }
    }

    /// Returns a number of actuators/controls
    pub fn nu(&self) -> usize {
        unsafe {
            let mj_model = self;
            (*mj_model.ptr()).nu as usize
        }
    }

    /// Returns a number of bodies in the model
    pub fn nbody(&self) -> usize {
        unsafe {
            let mj_model = self;
            (*mj_model.ptr()).nbody as usize
        }
    }

    /// number of fields in sensor data vector
    pub fn nsensordata(&self) -> usize {
        unsafe {
            let mj_model = self;
            (*mj_model.ptr()).nsensordata as usize
        }
    }

    /// Returns a number of meshes
    pub fn nmesh(&self) -> usize {
        unsafe {
            let mj_model = self;
            (*mj_model.ptr()).nmesh as usize
        }
    }

    /// Returns a number of vertices in all meshes
    pub fn nmeshvert(&self) -> usize {
        unsafe {
            let mj_model = self;
            (*mj_model.ptr()).nmeshvert as usize
        }
    }

    // Returns start index of vertices for a given mesh
    pub fn mesh_vertadr(&self, mesh_id: usize) -> usize {
        unsafe {
            let mj_model = self;
            (*(*mj_model.ptr()).mesh_vertadr.add(mesh_id)) as usize
        }
    }

    // Returns number of vertices for a given mesh
    pub fn mesh_vertnum(&self, mesh_id: usize) -> usize {
        unsafe {
            let mj_model = self;
            (*(*mj_model.ptr()).mesh_vertnum.add(mesh_id)) as usize
        }
    }

    // Returns start index of faces for a given mesh
    pub fn mesh_faceadr(&self, mesh_id: usize) -> usize {
        unsafe {
            let mj_model = self;
            (*(*mj_model.ptr()).mesh_faceadr.add(mesh_id)) as usize
        }
    }

    // Returns number of faces for a given mesh
    pub fn mesh_facenum(&self, mesh_id: usize) -> usize {
        unsafe {
            let mj_model = self;
            (*(*mj_model.ptr()).mesh_facenum.add(mesh_id)) as usize
        }
    }

    /// Returns a number of triangular faces in all meshes
    pub fn nmeshface(&self) -> usize {
        unsafe {
            let mj_model = self;
            (*mj_model.ptr()).nmeshface as usize
        }
    }

    /// Returns list of meshes in a scene
    pub fn meshes(&self) -> Vec<Mesh> {
        let mut meshes = Vec::new();
        let mj_model = self;
        let mj_model = unsafe { *mj_model.ptr() };

        for i in 0..self.nmesh() {
            let vertadr = self.mesh_vertadr(i);
            let vertnum = self.mesh_vertnum(i);
            let faceadr = self.mesh_faceadr(i);
            let facenum = self.mesh_facenum(i);

            // mesh data
            let vertices = extract_mesh_attribute(mj_model.mesh_vert, vertadr, vertnum);
            let normals =
                extract_mesh_attribute(mj_model.mesh_normal, vertadr, vertnum);
            let indices = extract_indices(mj_model.mesh_face, faceadr, facenum);

            // metadata
            let mesh_name_idx = unsafe { *mj_model.name_meshadr.add(i) as usize };
            let name = unsafe {
                CStr::from_ptr(mj_model.names.add(mesh_name_idx))
                    .to_str()
                    .to_owned()
                    .unwrap()
            };

            let mujoco_mesh = Mesh {
                vertices,
                normals,
                indices,
                name: String::from(name),
            };

            meshes.push(mujoco_mesh);
        }
        meshes
    }

    /// Get geoms of the model
    pub fn geoms(&self) -> Vec<Geom> {
        let mj_model = self;
        let mj_model = unsafe { *mj_model.ptr() };
        let n_geom = self.ngeom();

        let mut geoms: Vec<Geom> = Vec::new();
        let meshes = self.meshes();

        let body_pos_vec: Vec<f64> =
            extract_vector_float(mj_model.geom_pos as *mut Local<f64>, 3, n_geom)
                .iter()
                .map(|e| e.to_f64())
                .collect();
        let body_quat_vec: Vec<f64> =
            extract_vector_float(mj_model.geom_quat as *mut Local<f64>, 4, n_geom)
                .iter()
                .map(|e| e.to_f64())
                .collect();
        let body_size_vec: Vec<f64> =
            extract_vector_float(mj_model.geom_size as *mut Local<f64>, 4, n_geom)
                .iter()
                .map(|e| e.to_f64())
                .collect();
        let body_rgba_vec: Vec<f64> =
            extract_vector_float(mj_model.geom_rgba as *mut Local<f32>, 4, n_geom)
                .iter()
                .map(|e| e.to_f64())
                .collect();

        for i in 0..n_geom {
            // position
            let pos_array = body_pos_vec[i * 3..i * 3 + 3].to_vec();
            let pos_array: ArrayVec<f64, 3> = pos_array.into_iter().collect();
            let pos_array: [f64; 3] = pos_array.into_inner().unwrap();

            // quaternion
            let quat_array = body_quat_vec[i * 4..i * 4 + 4].to_vec();
            let quat_array: ArrayVec<f64, 4> = quat_array.into_iter().collect();
            let quat_array: [f64; 4] = quat_array.into_inner().unwrap();

            // size
            let size_array = body_size_vec[i * 3..i * 3 + 3].to_vec();
            let size_array: ArrayVec<f64, 3> = size_array.into_iter().collect();
            let size_array: [f64; 3] = size_array.into_inner().unwrap();

            // color
            let color_array = body_rgba_vec[i * 4..i * 4 + 4].to_vec();
            let color_array: ArrayVec<f64, 4> = color_array.into_iter().collect();
            let color_array: [f64; 4] = color_array.into_inner().unwrap();
            let color_array: [f32; 4] = [
                color_array[0] as f32,
                color_array[1] as f32,
                color_array[2] as f32,
                color_array[3] as f32,
            ];

            let mut mesh: Option<Mesh> = None;
            let mesh_id = unsafe { *mj_model.geom_dataid.add(i) };
            if mesh_id != -1 {
                mesh = Some(meshes[mesh_id as usize].clone());
            }

            // name
            let geom_name_idx = unsafe { *mj_model.name_geomadr.add(i) as usize };

            let name = unsafe {
                CStr::from_ptr(mj_model.names.add(geom_name_idx))
                    .to_str()
                    .to_owned()
                    .unwrap()
            };

            let geom_body = unsafe {
                Geom {
                    id: i as i32,
                    geom_type: geom_type_from(*mj_model.geom_type.add(i) as usize),
                    body_id: *mj_model.geom_bodyid.add(i),
                    geom_group: *mj_model.geom_group.add(i),
                    geom_contype: *mj_model.geom_contype.add(i),
                    pos: Vector3::from(pos_array),
                    quat: Quaternion::new(
                        quat_array[0],
                        quat_array[1],
                        quat_array[2],
                        quat_array[3],
                    ),
                    size: Vector3::from(size_array),
                    color: color_array,
                    mesh,
                    name: String::from(name),
                }
            };

            geoms.push(geom_body);
        }
        geoms
    }

    /// Get bodies of the model
    pub fn bodies(&self) -> Vec<Body> {
        let mj_model = self;
        let mj_model = unsafe { *mj_model.ptr() };
        let n_body = self.nbody();

        let body_pos_vec: Vec<f64> =
            extract_vector_float(mj_model.body_pos as *mut Local<f64>, 3, n_body)
                .iter()
                .map(|e| e.to_f64())
                .collect();
        let body_quat_vec: Vec<f64> =
            extract_vector_float(mj_model.body_quat as *mut Local<f64>, 4, n_body)
                .iter()
                .map(|e| e.to_f64())
                .collect();
        let _body_ipos_vec: Vec<f64> =
            extract_vector_float(mj_model.body_ipos as *mut Local<f64>, 3, n_body)
                .iter()
                .map(|e| e.to_f64())
                .collect();
        let _body_iquat_vec: Vec<f64> =
            extract_vector_float(mj_model.body_iquat as *mut Local<f64>, 4, n_body)
                .iter()
                .map(|e| e.to_f64())
                .collect();

        let mut bodies: Vec<Body> = Vec::new();
        for i in 0..n_body {
            // position
            let pos_array = body_pos_vec[i * 3..i * 3 + 3].to_vec();
            let pos_array: ArrayVec<f64, 3> = pos_array.into_iter().collect();
            let pos_array: [f64; 3] = pos_array.into_inner().unwrap();

            // quaternion
            let quat_array = body_quat_vec[i * 4..i * 4 + 4].to_vec();
            let quat_array: ArrayVec<f64, 4> = quat_array.into_iter().collect();
            let quat_array: [f64; 4] = quat_array.into_inner().unwrap();

            // metadata
            let name_idx = unsafe { *mj_model.name_bodyadr.add(i) as usize };

            let name = unsafe {
                CStr::from_ptr(mj_model.names.add(name_idx))
                    .to_str()
                    .to_owned()
                    .unwrap()
            };

            let geom_body = unsafe {
                Body {
                    id: i as i32,
                    parent_id: *mj_model.body_parentid.add(i),
                    geom_n: *mj_model.body_geomnum.add(i),
                    geom_addr: *mj_model.body_geomadr.add(i),
                    pos: Vector3::from(pos_array),
                    quat: Quaternion::new(
                        quat_array[0],
                        quat_array[1],
                        quat_array[2],
                        quat_array[3],
                    ),
                    name: String::from(name),
                }
            };

            bodies.push(geom_body);
        }
        bodies
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
    if model_ptr.is_null() {
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
        let m = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();

        // Check expected values
        check_expected_ids(&m);
    }

    #[test]
    fn from_xml_str() {
        let model_xml = Model::from_xml_str(*SIMPLE_XML).unwrap();
        // let model_file = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
        // assert_eq!(model_xml.to_vec(), model_file.to_vec());
        check_expected_ids(&model_xml);
    }

    #[test]
    fn from_bytes() {
        let model_xml = Model::from_xml(&*SIMPLE_XML_PATH).unwrap(); // _str
        let model_xml_bytes = model_xml.to_vec();
        let model_from_bytes = Model::from_bytes(&model_xml_bytes);
        assert_eq!(model_from_bytes.to_vec(), model_xml_bytes);
    }

    #[test]
    fn serialize() {
        let m = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
        let serialized = m.to_vec();
        assert_eq!(serialized.len(), unsafe {
            mujoco_rs_sys::no_render::mj_sizeModel(m.ptr)
        } as usize);
    }

    #[test]
    fn name_to_id() {
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

    #[test]
    fn geoms() {
        let model = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
        let geoms = model.bodies();

        assert!(!geoms.is_empty());
    }

    #[test]
    fn bodies() {
        let model = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
        let bodies = model.bodies();

        assert!(bodies.len() == 2);
    }

    #[test]
    fn meshes() {
        let model = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
        let meshes = model.meshes();

        assert!(meshes.len() == model.nmesh());
    }

    #[test]
    fn names() {
        let model = Model::from_xml(&*SIMPLE_XML_PATH).unwrap();
        let names = model.names();

        assert!(!names.is_empty());
    }
}
