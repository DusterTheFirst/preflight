pub use preflight_macros::avionics_harness;

#[repr(C)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Sensors;

#[repr(C)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Control {
    tvc: ThrustVector,
    // TODO: pyro
}

#[repr(C)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ThrustVector {
    x: f64,
    z: f64,
}

pub trait Avionics: Default {
    fn guide(&mut self, sensors: Sensors) -> Control;
}
