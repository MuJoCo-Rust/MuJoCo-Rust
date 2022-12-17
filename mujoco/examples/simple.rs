use mujoco_rust::{Model, Simulation};

const MJCF: &str = r#"<mujoco>
<worldbody>
    <light name="light0" diffuse=".5 .5 .5" pos="0 0 3" dir="0 0 -1"/>
    <geom name="geom0" type="plane" size="1 1 0.1" rgba=".9 0 0 1"/>
    <body name="body1" pos="0 0 1">
        <joint name="joint0" type="free"/>
        <geom name="geom1" type="box" size=".1 .2 .3" rgba="0 .9 0 1"/>
    </body>
</worldbody>
</mujoco>"#;

fn main() {
    let model = Model::from_xml_str(MJCF).unwrap();
    let simulation = Simulation::new(model);

    for _ in 0..1000 {
        println!(
            "time: {}, xpos: {:?}",
            simulation.state.time(),
            simulation.xpos()
        );
        simulation.step();
    }
}
