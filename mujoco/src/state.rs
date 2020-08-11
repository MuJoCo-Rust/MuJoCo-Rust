use crate::Model;

/// The time-dependent state of a MuJoCo simulation. Analagous to mjData from
/// the C API
#[derive(Debug)]
pub struct State<'a> {
    ptr: *mut mujoco_sys::no_render::mjData,
    // TODO: Perhaps this shouldn't be here, and we should manage cloning and resetting
    // in the `Sim` struct
    model: &'a Model,
}
impl<'a> State<'a> {
    /// Creates a new `State` from a [`Model`]
    pub fn new(model: &'a crate::Model) -> Self {
        let ptr = unsafe { mujoco_sys::no_render::mj_makeData(model.ptr) };
        assert_ne!(ptr, std::ptr::null_mut());
        // Do one forward step to initialize all fields
        // TODO: Double check this is necessary
        unsafe { mujoco_sys::no_render::mj_forward(model.ptr, ptr) };
        Self { ptr, model }
    }

    /// Resets the state to the default values
    // TODO: What does default values mean in this context?
    pub fn reset(&mut self) {
        unsafe { mujoco_sys::no_render::mj_resetData(self.model.ptr, self.ptr) }
        // Do one forward step to initialize all fields
        // TODO: Double check this is necessary
        unsafe { mujoco_sys::no_render::mj_forward(self.model.ptr, self.ptr) };
    }

    /// Gets the model associated with this `State`
    pub fn model(&self) -> &'a Model {
        self.model
    }
}
impl<'a> Drop for State<'a> {
    fn drop(&mut self) {
        unsafe { mujoco_sys::no_render::mj_deleteData(self.ptr) }
    }
}
impl<'a> Clone for State<'a> {
    fn clone(&self) -> Self {
        let ptr = unsafe {
            mujoco_sys::no_render::mj_copyData(
                std::ptr::null_mut(),
                self.model.ptr,
                self.ptr,
            )
        };
        assert_ne!(ptr, std::ptr::null_mut());
        // TODO: do we need to do mj_forward()?
        Self {
            ptr,
            model: self.model,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::SIMPLE_XML;
    use crate::{activate, Model};

    use super::*;

    #[test]
    fn new() {
        activate();
        let model = Model::from_xml_str(*SIMPLE_XML).unwrap();
        let state = State::new(&model);
        assert_ne!(state.ptr, std::ptr::null_mut());
        // TODO: Confirm the initial state is as expected
    }
}
