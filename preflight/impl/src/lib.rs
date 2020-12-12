#![no_std]

use uom::si::f64::Length;

pub use no_panic::no_panic;
pub use preflight_macros::avionics_harness;
pub use uom;

#[repr(C)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Sensors {
    pub altitude: Length,
}

#[repr(C)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Control {
    ABORT(AbortCause),
    Guidance(Guidance),
}

#[repr(C)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum AbortCause {
    ControlFailure = 0,
}

#[repr(C)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Guidance {
    pub tvc: ThrustVector,
    // TODO: pyro
}

#[repr(C)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ThrustVector {
    x: f64,
    z: f64,
}

pub trait Avionics {
    fn guide(&mut self, sensors: &Sensors) -> Option<Control>;
}
