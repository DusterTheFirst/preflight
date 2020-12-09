#![no_std]

use preflight_impl::{avionics_harness, Avionics, Control, Sensors};

pub struct Controller {
    ticks: u64,
}

impl Controller {
    const fn new() -> Self {
        Controller { ticks: 0 }
    }
}

#[avionics_harness(default = "Controller::new", no_panic = false)]
impl Avionics for Controller {
    fn guide(&mut self, sensors: &Sensors) -> Option<Control> {
        todo!()
        // Ok()
    }
}
