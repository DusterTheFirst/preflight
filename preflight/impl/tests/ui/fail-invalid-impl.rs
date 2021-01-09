#![no_std]

use preflight_impl::{avionics_harness, Avionics, Control, Sensors};

#[derive(Debug)]
struct Controller;

impl Controller {
    const fn new() -> Self {
        Controller
    }
}

#[avionics_harness(default = "Controller::new")]
impl !Avionics for Controller {
    fn guide(&mut self, _: &Sensors) -> Option<Control> {
        todo!()
    }
}

fn main() {}
