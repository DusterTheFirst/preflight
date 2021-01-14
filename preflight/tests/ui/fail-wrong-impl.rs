use preflight::{avionics_harness};

#[derive(Debug)]
struct Controller;

impl Controller {
    const fn new() -> Self {
        Controller
    }
}

trait Cool {}

#[avionics_harness(default = "Controller::new()")]
impl Cool for Controller {}

fn main() {}
