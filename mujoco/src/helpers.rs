use std::convert::TryInto;
use std::ffi::CString;

pub fn convert_err_buf(err_buf: Vec<u8>) -> String {
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
    err_str.into_string().expect("`CString` was not UTF-8!")
}

pub(crate) struct Local<T>(T);

/// Generic type for c float and double
pub trait LocalFloat {
    fn to_f64(&self) -> f64;
    fn to_f32(&self) -> f32;
}
impl LocalFloat for &*mut Local<f32> {
    fn to_f32(&self) -> f32 {
        unsafe { (***self).0 }
    }

    fn to_f64(&self) -> f64 {
        unsafe { (***self).0 as f64 }
    }
}

impl LocalFloat for &*mut Local<f64> {
    fn to_f32(&self) -> f32 {
        unsafe { (***self).0 as f32 }
    }

    fn to_f64(&self) -> f64 {
        unsafe { (***self).0 }
    }
}

impl LocalFloat for Local<f32> {
    fn to_f32(&self) -> f32 {
        self.0
    }

    fn to_f64(&self) -> f64 {
        self.0 as f64
    }
}

impl LocalFloat for Local<f64> {
    fn to_f32(&self) -> f32 {
        self.0 as f32
    }

    fn to_f64(&self) -> f64 {
        self.0
    }
}

/// Extract vertices and normals from a MuJoCo mesh
pub(crate) fn extract_mesh_attribute(
    array: *mut f32,
    offset: usize,
    count: usize,
) -> Vec<[f32; 3]> {
    let mut points: Vec<[f32; 3]> = vec![];

    let point_array: Vec<f32> = extract_vector_float(
        unsafe { array.add(offset * 3) } as *mut Local<f32>,
        3,
        count,
    )
    .iter()
    .map(|e| e.to_f32())
    .collect();

    for p in point_array.chunks(3) {
        let p: [f32; 3] = p.try_into().unwrap();
        points.push(p);
    }

    points
}

/// Extract normals from a MuJoCo mesh
pub(crate) fn extract_indices(
    array: *mut i32,
    face_addr: usize,
    face_num: usize,
) -> Vec<u32> {
    let mut indices: Vec<u32> = vec![];
    for j in face_addr..face_addr + face_num {
        unsafe {
            let face = array.add(j * 3);
            indices.push(*face.add(0) as u32);
            indices.push(*face.add(1) as u32);
            indices.push(*face.add(2) as u32);
        }
    }
    indices
}

/// Copy MuJoCo float/double array into Vec<T>
pub(crate) fn extract_vector_float<T>(
    vec: *mut T,
    element_size: usize,
    n_entries: usize,
) -> Vec<*mut T>
where
    T: LocalFloat,
{
    let mut result_vec: Vec<*mut T> = Vec::new();

    for i in 0..n_entries * element_size {
        let entry = unsafe { vec.add(i) };
        result_vec.push(entry);
    }

    result_vec
}
