#![no_std]
#![forbid(unsafe_code)]

use core::f32::consts::TAU;

use preflight::{
    avionics_harness,
    micromath::F32Ext,
    uom::si::angle::{degree, Angle},
    Avionics, Control, Guidance, Sensors, ThrustVector,
};

#[derive(Debug)]
pub struct Controller;

impl Controller {
    const fn new() -> Self {
        Controller
    }
}

#[avionics_harness(default = "Controller::new()")]
impl Avionics for Controller {
    fn guide(&mut self, sensors: &Sensors) -> Control {
        // Produce a sinusoidal TVC control

        // The angle of the seconds hand
        let time_angle = sensors.running_time.value * TAU;

        Control::Guidance(Guidance {
            tvc: ThrustVector {
                x: Angle::new::<degree>(time_angle.sin()),
                z: Angle::new::<degree>(time_angle.cos()),
            },
        })
        // Some(c)
        // todo!()
    }
}
