use preflight_impl::{avionics_harness, Avionics, Control, Sensors};

#[derive(Default)]
struct Controller;

trait Cool {}

#[avionics_harness]
impl Cool for Controller {}

fn main() {}
