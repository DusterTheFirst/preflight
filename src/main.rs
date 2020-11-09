use data_log::DataLogger;
use kiss3d::{
    camera::ArcBall,
    light::Light,
    window::{State, Window},
};
use log::trace;
use nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use ncollide3d::shape::{Cuboid, ShapeHandle};
use nphysics3d::{
    algebra::Force3,
    force_generator::DefaultForceGeneratorSet,
    joint::DefaultJointConstraintSet,
    math::ForceType,
    object::{
        Body, BodyPartHandle, Collider, ColliderDesc, DefaultBodySet, DefaultColliderSet,
        RigidBody, RigidBodyDesc,
    },
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};
use timescale_data::timescale_data;

mod data_log;

#[timescale_data]
#[derive(Debug, Clone)]
struct VectorDatapoints {
    position: Vector3<f64>,
}

fn main() {
    let mut logger = DataLogger::<VectorDatapoints>::new().unwrap();

    // let mut window = Window::new("h");

    // window.set_background_color(0.0, 0.5, 1.0);
    // window.set_light(Light::Absolute(Point3::new(100.0, 1000.0, 300.0)));
    // window.set_light(Light::StickToCamera);
    // window.set_framerate_limit(Some(60));

    // let mut camera = ArcBall::new(Point3::new(5.0, 2.0, -1.0), Point3::origin());
    // camera.rebind_drag_button(None);
    // camera.rebind_rotate_button(None);
    // camera.rebind_reset_key(None);

    // let mut c = window.add_cube(1.0, 1.0, 1.0);
    // c.set_color(1.0, 0.0, 0.0);

    // window.add_cube(100.0, 1.0, 100.0).set_color(0.0, 1.0, 0.0);

    // PHYSICS
    let mut mechanical_world = DefaultMechanicalWorld::new(Vector3::y() * -9.81);
    let mut geometrical_world = DefaultGeometricalWorld::new();

    let mut bodies = DefaultBodySet::new();
    let mut colliders = DefaultColliderSet::new();
    let mut joint_constraints = DefaultJointConstraintSet::new();
    let mut force_generators = DefaultForceGeneratorSet::new();

    let rocket_body = RigidBodyDesc::new().mass(100.0).build();

    let rocket_body_handle = bodies.insert(rocket_body);

    let rocket_collider = ColliderDesc::new(ShapeHandle::new(Cuboid::new(Vector3::new(
        10.0, 10.0, 10.0,
    ))))
    .density(1.0)
    .build(BodyPartHandle(rocket_body_handle, 0));

    let rocket_collider_handle = colliders.insert(rocket_collider);

    let mut time: f64 = 0.0;

    loop
    /* window.render_with_camera(&mut camera) */
    {
        // c.prepend_to_local_rotation(&rot);
        // c.append_translation(&Translation3::new(0.0, up, 0.0));

        // Run the simulation.
        mechanical_world.step(
            &mut geometrical_world,
            &mut bodies,
            &mut colliders,
            &mut joint_constraints,
            &mut force_generators,
        );
        time += mechanical_world.timestep();

        let rocket_body: &mut RigidBody<f64> = bodies.rigid_body_mut(rocket_body_handle).unwrap();
        let rocket_collider: &mut Collider<f64, _> =
            colliders.get_mut(rocket_collider_handle).unwrap();

        rocket_body.apply_force(
            0,
            &Force3::new(Vector3::y() * 100.0, Vector3::new(0.0, 0.0, 0.0)),
            ForceType::Force,
            true,
        );

        logger
            .add_data_point(VectorDatapoints {
                time,
                position: dbg!(rocket_body.position().translation.vector),
            })
            .unwrap();

        // c.set_local_translation(Translation3::from(
        //     rocket_body.position().translation.vector.map(|x| x as f32),
        // ));

        // camera.look_at(
        //     Point3::new(100.0, 20.0, 0.0),
        //     Point3::from(c.data().local_translation().vector),
        // );
        // camera.set_dist(10.0);
    }
}
