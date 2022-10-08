pub use mujoco_sys::no_render::mjMAXVFS as MAX_FILES;
pub use mujoco_sys::no_render::mjMAXVFSNAME as MAX_FILENAME_LEN;

use std::ffi::{c_char, c_int};
use std::ffi::{CStr, CString};
use core::ops::Deref;
use std::borrow::Borrow;
use std::mem::size_of;

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

pub(crate) struct FileNameContainer(*const ::std::os::raw::c_char);

impl FileNameContainer
{
    fn get(&self, idx : usize) -> *const ::std::os::raw::c_char
    {
        println!("attempting to get file name at: {:?}", self.0);
        unsafe { self.0.offset((idx * mujoco_sys::no_render::mjMAXVFSNAME as usize) as isize) }
    }
}
/*
impl ::std::ops::Index<usize> for FileNameContainer
{
    type Output = *const ::std::os::raw::c_char;

    fn index(&self, idx: usize) -> &Self::Output
    {
        unsafe { &self.0.offset((idx * mujoco_sys::no_render::mjMAXVFSNAME as usize) as isize) }
    }
}*/

pub(crate) struct SizeContainer(*const ::std::os::raw::c_int);

impl SizeContainer
{
    fn get(&self, idx : usize) -> ::std::os::raw::c_int
    {
        //unsafe { (*self.0).into() }//*self.0.offset(idx as isize) }
        let size_ptr : *const ::std::os::raw::c_int = unsafe {self.0.offset((idx as usize) as isize)};
        println!("attempting to access size pointer at: {:?}", size_ptr);
        let size_ = unsafe {*size_ptr};
        println!("Success!");
        size_
    }
}
/*
impl ::std::ops::Index<usize> for SizeContainer
{
    type Output = *const ::std::os::raw::c_int;

    fn index(&self, idx: usize) -> &Self::Output
    {
        unsafe { &self.0.offset((idx * mujoco_sys::no_render::mjMAXVFSNAME as usize) as isize) }
    }
}*/

pub(crate) struct DataContainer(*mut ::std::os::raw::c_void);

impl DataContainer
{
    fn get(&self, idx : usize) -> *const ::std::os::raw::c_void
    {
        println!("Attempting to get data at index {}", idx);
        let ret = unsafe { self.0.offset((idx as usize) as isize) };
        println!("Success!");
        ret
    }
}
/*impl ::std::ops::Index<usize> for DataContainer
{
    type Output = *mut ::std::os::raw::c_void;

    fn index(&self, idx: usize) -> &Self::Output
    {
        unsafe { &self.0.offset((idx * mujoco_sys::no_render::mjMAXVFSNAME as usize) as isize) }
    }
}*/


pub struct VfsWrapper
{
    pub(crate) mem : *const mujoco_sys::no_render::mjVFS,
    pub(crate) nfile : *const ::std::os::raw::c_int,
    pub(crate) filename : FileNameContainer,
    pub(crate) filesize : SizeContainer,//*const ::std::os::raw::c_int,
    pub(crate) filedata : DataContainer,//*mut ::std::os::raw::c_void,
}

const filename_offset : isize = size_of::<::std::os::raw::c_int>() as isize;
const filesize_offset : isize = filename_offset + size_of::<[[c_char; 1000usize]; 2000usize]>()/*(size_of::<::std::os::raw::c_char>()
                                                  * mujoco_sys::no_render::mjMAXVFSNAME as usize
                                                  * mujoco_sys::no_render::mjMAXVFS as usize
                                                  )*/ as isize;

const data_offset : isize = 4 + filesize_offset + size_of::<[c_int; 2000usize]>()/*(size_of::<::std::os::raw::c_int>()
                                                                 * mujoco_sys::no_render::mjMAXVFS as usize)*/ as isize;

impl Default for VfsWrapper
{
    fn default() -> Self
    {
        let pool = unsafe { std::alloc::alloc(std::alloc::Layout::new::<mujoco_sys::no_render::mjVFS>()) };
        let n = pool as *const ::std::os::raw::c_int;
        let filename_ptr: *const u8 = unsafe {pool.offset(filename_offset)};
        let filesize_ptr: *const u8 = unsafe {pool.offset(filesize_offset)};
        let data_ptr: *const u8 = unsafe {pool.offset(data_offset)};

        println!("base pointer: {:?}",pool);
        println!("name pointer: {:?}",filename_ptr);
        println!("size pointer: {:?}",filesize_ptr);
        println!("data pointer: {:?}",data_ptr);

        unsafe { *filesize_ptr };
        unsafe{mujoco_sys::no_render::mj_makeEmptyFileVFS(pool as *mut _, "I'm a tulip".as_ptr() as *const _, 1);}
        unsafe { println!("{}",*filesize_ptr) };

        Self
        {
            mem: pool as *const mujoco_sys::no_render::mjVFS,
            nfile : n,
            filename : FileNameContainer(filename_ptr as *const ::std::os::raw::c_char),
            filesize : SizeContainer(filesize_ptr as *const ::std::os::raw::c_int),
            filedata : DataContainer(data_ptr as *mut ::std::os::raw::c_void)
        }
    }
}

pub struct Vfs {
    pub(crate) vfs: VfsWrapper,
}

impl Deref for VfsWrapper
{
    type Target = *const mujoco_sys::no_render::mjVFS;

    fn deref(&self) -> &Self::Target
    {
        &self.mem
    }
}

impl Into<*const mujoco_sys::no_render::mjVFS> for VfsWrapper
{
    fn into(self) -> *const mujoco_sys::no_render::mjVFS
    {
        unsafe { self.mem }
    }
}

impl AsRef<*const mujoco_sys::no_render::mjVFS> for VfsWrapper
{
    fn as_ref(&self) -> &*const mujoco_sys::no_render::mjVFS
    {
        &self.mem
    }
}

impl Vfs {
    /// Initializes a new empty `Vfs`
    pub fn new() -> Self {
        Self::default()
    }

    /// Looks up the index of the file from the filename
    fn get_idx(&self, filename: &CStr) -> Option<usize> {
        let idx = unsafe {
            mujoco_sys::no_render::mj_findFileVFS(self.vfs.mem, filename.as_ptr())
        };
        if idx == -1 {
            None
        } else {
            debug_assert!(idx < unsafe { *self.vfs.nfile});
            Some(idx as usize)
        }
    }

    /// Gets a file's contents from the `Vfs`
    pub fn get_file(&self, filename: &str) -> Option<&[u8]> {
        let idx = self.get_idx(&CString::new(filename).unwrap())?;
        let file_size = self.vfs.filesize.get(idx) as usize;
        let start_ptr = self.vfs.filedata.get(idx) as *const u8;
        println!("start_ptr: {:?}", start_ptr);
        println!("size: {}", file_size);
        Some(unsafe { std::slice::from_raw_parts(start_ptr, file_size) })
    }   

    /// Deletes a file from the `Vfs` if it exists, and returns if such a file
    /// was found
    pub fn delete_file(&mut self, filename: &str) -> bool {
        let c_str = CString::new(filename).unwrap();
        let result = unsafe {
            mujoco_sys::no_render::mj_deleteFileVFS(self.vfs.mem as *mut _, c_str.as_ptr())
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
                self.vfs.mem as *mut _,
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
        let start_ptr = self.vfs.filedata.get(idx) as *mut u8;
        println!("Start pointer: {:?}", start_ptr);
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
        unsafe { mujoco_sys::no_render::mj_defaultVFS(result.vfs.mem as *mut _) };
        result
    }
}
impl Drop for Vfs {
    fn drop(&mut self) {
        unsafe { mujoco_sys::no_render::mj_deleteVFS(self.vfs.mem as *mut _); }
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
    fn add_contents() {
        let filename = "asdf/dsdfs$@f.123";
        let content = "3klj032#$>>😮f";
        {
        let mut vfs = Vfs::new();
        println!("Attempting to add a file with contents");
        vfs.add_file(filename, content.as_bytes()).unwrap();
        assert_eq!(vfs.get_file(filename).unwrap(), content.as_bytes());
        }
        println!("Was able to add file");
    }

    #[test]
    fn add_multiple() {
        let filename = "asdf/dsdfs$@f.123";
        let content = "3klj032#$>>😮f";
        let mut vfs = Vfs::new();
        println!("Attempting to add a file with contents");
        vfs.add_file(filename, content.as_bytes()).unwrap();
        assert_eq!(vfs.get_file(filename).unwrap(), content.as_bytes());
        println!("Was able to add file");
        let filename2 = "bsdf/esdfs$@f.123";
        let content2 = "58lj032#$>>😮f";
        println!("Attempting to add a file with contents");
        vfs.add_file(filename2, content2.as_bytes()).unwrap();
        assert_eq!(vfs.get_file(filename2).unwrap(), content2.as_bytes());
        println!("Was able to add file");
        let filename3 = "bsdf/fsdfs$@f.123";
        let content3 = "58lj098032#$>>😮f";
        println!("Attempting to add a file with contents");
        vfs.add_file(filename3, content3.as_bytes()).unwrap();
        assert_eq!(vfs.get_file(filename3).unwrap(), content3.as_bytes());
        println!("Was able to add file");
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
        println!("Trying to delete file");
        assert_eq!(vfs.delete_file(filename), true);
        println!("Done");
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
