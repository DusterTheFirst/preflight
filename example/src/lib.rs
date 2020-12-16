#![no_std]

use core::panic::PanicInfo;

use preflight_impl::{avionics_harness, Avionics, Control, Sensors};

// avionics_panic!();

#[panic_handler]
fn handle_panic(_: &PanicInfo) -> ! {
    loop {}
}

pub struct Controller {
    ticks: u64,
}

impl Controller {
    const fn new() -> Self {
        Controller { ticks: 0 }
    }
}

#[avionics_harness(default = "Controller::new", penis = "", panic_handler = false)]
impl Avionics for Controller {
    fn guide(&mut self, sensors: &Sensors) -> Option<Control> {
        None
        // Some(c)
    }
}
