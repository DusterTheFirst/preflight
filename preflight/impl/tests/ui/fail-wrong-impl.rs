use preflight_impl::{avionics_harness, Avionics, Control, Sensors};

struct Controller;

impl Controller {
    const fn new() -> Self {
        Controller
    }
}

trait Cool {}

#[avionics_harness(default = "Controller::new")]
impl Cool for Controller {}

fn main() {}