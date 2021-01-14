use preflight::{avionics_harness, Avionics, Control, Sensors};

#[derive(Debug)]
struct Controller;

impl Controller {
    const fn new() -> Self {
        Controller
    }
}

#[avionics_harness(default = "Controller::new()", arg = "ument")]
impl Avionics for Controller {
    fn guide(&mut self, _: &Sensors) -> Control {
        todo!()
    }
}

fn main() {}
