#![no_std]
#![forbid(unsafe_code)]

use preflight::{Avionics, Control, Guidance, Sensors, ThrustVector, avionics_harness, uom::si::angle::{Angle, degree}};

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

        Control::Guidance(Guidance {
            tvc: ThrustVector {
                x: Angle::new::<degree>(sensors.running_time.),
                z: Angle::new::<degree>(0.0),
            }
        })
        // Some(c)
        // todo!()
    }
}
