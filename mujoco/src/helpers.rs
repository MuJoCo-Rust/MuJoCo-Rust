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
