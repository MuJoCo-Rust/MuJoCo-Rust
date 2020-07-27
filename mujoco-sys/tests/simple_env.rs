use lazy_static::lazy_static;
use mujoco_sys::*;
use std::ffi::{CStr, CString};

const xml: &str = r#"
<mujoco>
   <worldbody>
      <light diffuse=".5 .5 .5" pos="0 0 3" dir="0 0 -1"/>
      <geom type="plane" size="1 1 0.1" rgba=".9 0 0 1"/>
      <body pos="0 0 1">
         <joint type="free"/>
         <geom type="box" size=".1 .2 .3" rgba="0 .9 0 1"/>
      </body>
   </worldbody>
</mujoco>
"#;

lazy_static! {
    static ref MJ_HOME: CString = CString::new(
        dirs::home_dir()
            .expect("Could not locate home directory!")
            .join(".mujoco")
            .join("mujoco200")
            .to_str()
            .unwrap()
    )
    .unwrap();
    static ref MJ_KEY: CString = CString::new(
        dirs::home_dir()
            .expect("Could not locate home directory!")
            .join(".mujoco")
            .join("mjkey.txt")
            .to_str()
            .unwrap()
    )
    .unwrap();
}

#[test]
fn test_activate() {
    assert_eq!(unsafe { mj_activate(MJ_KEY.as_ptr()) }, 1);
}
