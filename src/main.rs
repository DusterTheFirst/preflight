// use std::{
//     io::{stdout, Write},
//     thread,
//     time::Duration,
// };

// use log::trace;
// use nalgebra::Vector3;
// use ncollide3d::shape::{Cuboid, ShapeHandle};
// use nphysics3d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};
// use nphysics3d::{
//     algebra::Force3,
//     math::ForceType,
//     object::{DefaultBodySet, DefaultColliderSet, RigidBodyDesc},
// };
// use nphysics3d::{
//     force_generator::DefaultForceGeneratorSet,
//     object::{Body, ColliderDesc},
// };
// use nphysics3d::{joint::DefaultJointConstraintSet, object::BodyPartHandle};

// fn main() {
//     dotenv::dotenv().ok();
//     pretty_env_logger::init();

//     let mut mechanical_world = DefaultMechanicalWorld::new(Vector3::y() * -9.81);
//     let mut geometrical_world = DefaultGeometricalWorld::new();

//     let mut bodies = DefaultBodySet::new();
//     let mut colliders = DefaultColliderSet::new();
//     let mut joint_constraints = DefaultJointConstraintSet::new();
//     let mut force_generators = DefaultForceGeneratorSet::new();

//     let rocket_body = RigidBodyDesc::new().mass(100.0).build();

//     let rocket_body_handle = bodies.insert(rocket_body);

//     let rocket_collider = ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector3::new(
//         10.0, 10.0, 10.0,
//     ))))
//     .density(1.0)
//     .build(BodyPartHandle(rocket_body_handle, 0));

//     let rocket_collider_handle = colliders.insert(rocket_collider);

//     loop {
//         // Run the simulation.
//         mechanical_world.step(
//             &mut geometrical_world,
//             &mut bodies,
//             &mut colliders,
//             &mut joint_constraints,
//             &mut force_generators,
//         );

//         rocket_body.apply_force(
//             0,
//             Force3::new(Vector3::y() * 10, Vector3::new(0.0, 0.0, 0.0)),
//             ForceType::Force,
//             true,
//         );

//         let rocket: &dyn Body<f64> = bodies.get(rocket_body_handle).unwrap();
//         print!("\x1B[2J\x1B[1;1H");
//         stdout().flush().ok();
//         trace!(
//             "v={} a={} ∆t={}",
//             rocket.generalized_velocity(),
//             rocket.generalized_acceleration(),
//             mechanical_world.timestep(),
//         );
//         stdout().flush().ok();

//         thread::sleep(Duration::from_millis(500));
//     }
// }

use nalgebra::{Vector3, UnitQuaternion};
use kiss3d::window::Window;
use kiss3d::light::Light;

fn main() {
    let mut window = Window::new("Kiss3d: cube");
    let mut c      = window.add_cube(1.0, 1.0, 1.0);

    c.set_color(1.0, 0.0, 0.0);

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render() {
        c.prepend_to_local_rotation(&rot);
    }
} 