use data_log::DataLogger;
use kiss3d::{camera::ArcBall, light::Light, text::Font, widget_ids, window::Window};
use log::{error, info, warn, LevelFilter};
use nalgebra::{DVectorSlice, Point2, Point3, Translation3, Vector3};
use ncollide3d::shape::{Cuboid, ShapeHandle};
use nphysics3d::{
    algebra::Force3,
    force_generator::DefaultForceGeneratorSet,
    joint::DefaultJointConstraintSet,
    math::ForceType,
    object::{
        Body, BodyPartHandle, ColliderDesc, DefaultBodySet, DefaultColliderSet, RigidBodyDesc,
    },
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};
use simplelog::{Config, TermLogger, TerminalMode};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use timescale::ToTimescale;
use ui::{gui, theme, Ids};

mod data_log;
mod force_generator;
mod ui;

#[derive(Debug, Clone, ToTimescale)]
struct VectorDatapoints {
    position: Vector3<f64>,
}

// TODO: anyhow?
fn main() {
    // Setup stdout/err logging
    TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::Mixed).unwrap();

    // Setup csv logging
    let mut logger = DataLogger::<VectorDatapoints>::new().unwrap();

    /* // Create a window for graphics
    let mut window = Window::new("Simulation");
    window.set_background_color(0.0, 0.5, 1.0);
    // window.set_light(Light::Absolute(Point3::new(100.0, 1000.0, 300.0)));
    window.set_light(Light::StickToCamera);
    window.set_framerate_limit(Some(60));

    // Ui
    let ids = Ids::new(window.conrod_ui_mut().widget_id_generator());
    window.conrod_ui_mut().theme = theme();

    let font_regular =
        Font::from_bytes(include_bytes!("../fonts/SourceCodePro-Regular.ttf")).unwrap();
    let font_bold = Font::from_bytes(include_bytes!("../fonts/SourceCodePro-Bold.ttf")).unwrap();

    // Create the camera to render from
    let mut camera = ArcBall::new(Point3::new(5.0, 2.0, -1.0), Point3::origin());
    camera.set_dist(100.0);
    // camera.rebind_drag_button(None);
    // camera.rebind_rotate_button(None);
    // camera.rebind_reset_key(None); */

    // Setup the physics objects
    let mut mechanical_world = DefaultMechanicalWorld::<f64>::new(Vector3::y() * -9.81);
    let mut geometrical_world = DefaultGeometricalWorld::<f64>::new();
    let mut bodies = DefaultBodySet::<f64>::new();
    let mut colliders = DefaultColliderSet::<f64>::new();
    let mut joint_constraints = DefaultJointConstraintSet::<f64>::new();
    let mut force_generators = DefaultForceGeneratorSet::<f64>::new();

    // Create the rocket physics object
    let rocket_body_handle = bodies.insert(
        RigidBodyDesc::new()
            .translation(Vector3::new(0.0, 100.0, 0.0))
            .mass(0.100)
            .build(),
    );
    let rocket_shape = Cuboid::new(Vector3::new(10.0, 10.0, 10.0));
    let rocket_collider_handle = colliders.insert(
        ColliderDesc::new(ShapeHandle::new(rocket_shape))
            .build(BodyPartHandle(rocket_body_handle, 0)),
    );

    /* // Create the visible rocket
    let mut visible_rocket = window.add_cube(
        (rocket_shape.half_extents[0] * 2.0) as f32,
        (rocket_shape.half_extents[1] * 2.0) as f32,
        (rocket_shape.half_extents[2] * 2.0) as f32,
    );
    visible_rocket.set_color(1.0, 0.0, 0.0); */

    // Create the ground physics object
    let ground_handle = bodies.insert(RigidBodyDesc::new().gravity_enabled(false).build());
    let ground_shape = Cuboid::new(Vector3::new(100.0, 1.0, 100.0));
    let ground_collider_handle = colliders.insert(
        ColliderDesc::new(ShapeHandle::new(ground_shape)).build(BodyPartHandle(ground_handle, 0)),
    );

    /* // Create the visible ground
    let mut ground = window.add_cube(
        (ground_shape.half_extents[0] * 2.0) as f32,
        (ground_shape.half_extents[1] * 2.0) as f32,
        (ground_shape.half_extents[2] * 2.0) as f32,
    );
    ground.set_color(0.0, 1.0, 0.0); */

    // Hold the elapsed time
    let mut elapsed_time: f64 = 0.0;

    // Allow simulation to be gracefully halted
    let stopped = Arc::new(AtomicBool::new(false));

    // Setup ctrl+c handler for graceful shutdown
    {
        let stopped = stopped.clone();

        if let Err(e) = ctrlc::set_handler(move || {
            stopped.store(true, Ordering::Relaxed);
            info!("The simulation will halt and flush the DataLogger...");
        }) {
            error!("Error: {}", e);
            warn!(indoc::indoc! {"
                The ctrl+c handler was unable to be set. \
                The outputted csv file for this run may be cut short and corrupted when you use ctrl+c to shut down the main loop \
            "});
        }
    }

    // The simulation loop
    while !stopped.load(Ordering::Relaxed)
    /* && window.render_with_camera(&mut camera) */
    {
        // Physics
        {
            // Apply the thrust force to the rocket
            let rocket_body = bodies.rigid_body_mut(rocket_body_handle).unwrap();
            let rocket_collider = colliders.get_mut(rocket_collider_handle).unwrap();

            rocket_body.apply_force(
                0,
                &Force3::new(Vector3::new(0.0, 10.0, 20.0), Vector3::new(0.0, 0.0, 0.0)),
                ForceType::Force,
                true,
            );
        }

        // Run the simulation.
        mechanical_world.step(
            &mut geometrical_world,
            &mut bodies,
            &mut colliders,
            &mut joint_constraints,
            &mut force_generators,
        );
        // Increment the elapsed time by the step that the mechanical world has taken
        elapsed_time += mechanical_world.timestep();

        let rocket_body = bodies.rigid_body_mut(rocket_body_handle).unwrap();

        // Logging
        {
            logger
                .add_data_point(
                    elapsed_time,
                    VectorDatapoints {
                        position: rocket_body.position().translation.vector,
                    },
                )
                .unwrap();
        }

        // Rendering
        /* {
            let rocket_position: Vector3<f32> =
                nalgebra::convert(rocket_body.position().translation.vector);
            let rocket_velocity: Vector3<f32> = nalgebra::convert(rocket_body.velocity().linear);
            let rocket_acceleration = rocket_body.generalized_acceleration().map(|x| x as f32);

            visible_rocket.set_local_translation(Translation3::from(rocket_position));

            camera.set_at(Point3::from(rocket_position));

            {
                let mut ui = window.conrod_ui_mut().set_widgets();
                gui(&mut ui, &ids);
            }

            window.draw_text(
                &format!(indoc::indoc! {"
                    Position
                        x:
                        y:
                        z:
                    Velocity
                        x:
                        y:
                        z:
                    Acceleration
                        x:
                        y:
                        z:
                "}),
                &Point2::new(0.0, 0.0),
                50.0,
                &font_bold,
                &Point3::new(1.0, 1.0, 1.0),
            );

            window.draw_text(
                &format!(
                    indoc::indoc! {"

                        {:+.5}
                        {:+.5}
                        {:+.5}

                        {:+.5}
                        {:+.5}
                        {:+.5}

                        {:+.5}
                        {:+.5}
                        {:+.5}
                    "},
                    rocket_position[0],
                    rocket_position[1],
                    rocket_position[2],
                    rocket_velocity[0],
                    rocket_velocity[1],
                    rocket_velocity[2],
                    rocket_acceleration[0],
                    rocket_acceleration[1],
                    rocket_acceleration[2]
                ),
                &Point2::new(150.0, 0.0),
                50.0,
                &font_regular,
                &Point3::new(1.0, 1.0, 1.0),
            )
        } */
    }

    if let Err(e) = logger.flush() {
        error!("Error: {}", e);
        warn!(indoc::indoc! {"
            The csv logger was unable to flush its output. \
            The outputted csv file for this run may be cut short or corrupted \
        "});
    }

    info!("Halted!");
}
