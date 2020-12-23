#![no_std]
#![forbid(unsafe_code)]

use preflight_impl::{avionics_harness, Avionics, Control, Sensors};

#[derive(Debug)]
pub struct Controller {
    ticks: u64,
}

impl Controller {
    const fn new() -> Self {
        Controller { ticks: 0 }
    }
}

#[avionics_harness(default = "Controller::new")]
impl Avionics for Controller {
    fn guide(&mut self, sensors: &Sensors) -> Option<Control> {
        self.ticks += 1;
        // None
        // Some(c)
        todo!()
    }
}
