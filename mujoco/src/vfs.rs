pub use mujoco_sys::no_render::mjMAXVFS as MAX_FILES;
pub use mujoco_sys::no_render::mjMAXVFSNAME as MAX_FILENAME_LEN;

use std::ffi::{CStr, CString};

/// An error when adding a file to a [`Vfs`] via [`Vfs::add_file()`]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum AddError {
    VfsFull,
    RepeatedName,
}
impl std::fmt::Display for AddError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for AddError {}

pub struct Vfs {
    vfs: mujoco_sys::no_render::mjVFS,
}
impl Vfs {
    /// Initializes a new empty `Vfs`
    pub fn new() -> Self {
        Self::default()
    }

    /// Looks up the index of the file from the filename
    fn get_idx(&self, filename: &CStr) -> Option<usize> {
        let idx = unsafe {
            mujoco_sys::no_render::mj_findFileVFS(&self.vfs, filename.as_ptr())
        };
        if idx == -1 {
            None
        } else {
            debug_assert!(idx < self.vfs.nfile);
            Some(idx as usize)
        }
    }

    /// Gets a file's contents from the `Vfs`
    pub fn get_file(&self, filename: &str) -> Option<&[u8]> {
        let idx = self.get_idx(&CString::new(filename).unwrap())?;
        let file_size = self.vfs.filesize[idx] as usize;
        let start_ptr = self.vfs.filedata[idx] as *const u8;
        Some(unsafe { std::slice::from_raw_parts(start_ptr, file_size) })
    }

    /// Deletes a file from the `Vfs` if it exists, and returns if such a file
    /// was found
    pub fn delete_file(&mut self, filename: &str) -> bool {
        let c_str = CString::new(filename).unwrap();
        let result = unsafe {
            mujoco_sys::no_render::mj_deleteFileVFS(&mut self.vfs, c_str.as_ptr())
        };
        debug_assert!(result == 0 || result == -1);
        result != -1
    }

    /// Adds a file to the `Vfs` from some given contents
    pub fn add_file(
        &mut self,
        filename: &str,
        contents: &[u8],
    ) -> Result<(), AddError> {
        let filename = CString::new(filename).unwrap();
        let file_size = contents.len();
        let add_errno = unsafe {
            mujoco_sys::no_render::mj_makeEmptyFileVFS(
                &mut self.vfs,
                filename.as_ptr(),
                file_size as std::os::raw::c_int,
            )
        };
        match add_errno {
            1 => Err(AddError::VfsFull),
            2 => Err(AddError::RepeatedName),
            0 => Ok(()),
            _ => unreachable!(),
        }?;

        let idx = self.get_idx(&filename).unwrap();
        let start_ptr = self.vfs.filedata[idx] as *mut u8;
        let file_slice =
            unsafe { std::slice::from_raw_parts_mut(start_ptr, file_size) };
        file_slice.copy_from_slice(contents);
        Ok(())
    }
}
impl Default for Vfs {
    fn default() -> Self {
        let mut result = Self {
            vfs: Default::default(),
        };
        unsafe { mujoco_sys::no_render::mj_defaultVFS(&mut result.vfs) };
        result
    }
}
impl Drop for Vfs {
    fn drop(&mut self) {
        unsafe { mujoco_sys::no_render::mj_deleteVFS(&mut self.vfs) }
    }
}

#[cfg(test)]
mod tests {
    use super::{AddError, Vfs};

    #[test]
    fn new() {
        Vfs::new();
    }

    #[test]
    fn add() {
        let filename = "asdf/dsdfs$@f.123";
        let content = "3klj032#$>>😮f";
        let mut vfs = Vfs::new();
        vfs.add_file(filename, content.as_bytes()).unwrap();
        assert_eq!(vfs.get_file(filename).unwrap(), content.as_bytes());
    }

    #[test]
    fn delete_without_add() {
        let mut vfs = Vfs::new();
        assert_eq!(vfs.delete_file("asdf"), false);
    }

    #[test]
    fn add_then_delete() {
        let filename = "file";
        let mut vfs = Vfs::new();
        vfs.add_file(filename, "asdf".as_bytes()).unwrap();
        assert_eq!(vfs.delete_file(filename), true);
    }

    #[test]
    fn add_twice_protected() {
        let filename = "file";
        let mut vfs = Vfs::new();
        vfs.add_file(filename, "asdf".as_bytes()).unwrap();
        let second_add_result = vfs.add_file(filename, "asdf".as_bytes());
        assert_eq!(second_add_result, Err(AddError::RepeatedName));
    }
}
