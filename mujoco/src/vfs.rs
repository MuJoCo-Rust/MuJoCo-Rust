pub struct Vfs {
    vfs: mujoco_sys::no_render::mjVFS,
}
impl Vfs {
    pub fn new() -> Self {
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
    use super::Vfs;

    #[test]
    fn new() {
        Vfs::new();
    }
}
