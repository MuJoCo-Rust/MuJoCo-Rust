use mujoco_rs_sys::mjData;

/// The time-dependent state of a MuJoCo simulation. Analagous to mjData from
/// the C API
#[derive(Debug)]
pub struct State {
    pub(crate) ptr: *mut mujoco_rs_sys::no_render::mjData,
}

unsafe impl Send for State {}
unsafe impl Sync for State {}

impl State {
    /// Creates a new `State` from a [`Model`]
    pub fn new(model: &crate::Model) -> Self {
        let ptr = unsafe { mujoco_rs_sys::no_render::mj_makeData(model.ptr) };
        assert_ne!(ptr, std::ptr::null_mut());
        // Do one forward step to initialize all fields
        // TODO: Double check this is necessary
        unsafe { mujoco_rs_sys::no_render::mj_forward(model.ptr, ptr) };
        Self { ptr }
    }

    /// Gets the low level [`mjData`] that the `Data` uses under the hood
    pub fn ptr(&self) -> *mut mjData {
        self.ptr
    }

    /// Simulation time in seconds
    pub fn time(&self) -> f64 {
        unsafe {
            let mj_data = self.ptr;
            (*mj_data).time
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe { mujoco_rs_sys::no_render::mj_deleteData(self.ptr) }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::SIMPLE_XML;
    use crate::Model;

    use super::*;

    #[test]
    fn new() {
        let model = Model::from_xml_str(*SIMPLE_XML).unwrap();
        let state = State::new(&model);
        assert_ne!(state.ptr, std::ptr::null_mut());
    }
}
