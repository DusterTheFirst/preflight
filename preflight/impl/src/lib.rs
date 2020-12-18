#![no_std]

use core::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, Mul, Neg, Rem, RemAssign, Sub, SubAssign},
};

pub use preflight_macros::avionics_harness;
pub use uom;
pub use uom::si::f64::{
    Acceleration, Angle, AngularAcceleration, AngularVelocity, Length, MagneticFluxDensity,
    Pressure,
};

#[repr(C)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Sensors {
    /// Calculated altitude
    pub altitude: Length,
    /// Three axis of linear acceleration data (acceleration minus gravity) in m/s^2
    pub linear_acceleration: Vector3<Acceleration>,
    /// Three axis of gravitational acceleration (minus any movement) in m/s^2
    pub gravity_acceleration: Vector3<Acceleration>,
    /// Three axis of acceleration (gravity + linear motion) in m/s^2
    pub both_acceleration: Vector3<Acceleration>,
    /// Three axis orientation data based on a 360° sphere
    pub orientation: Vector3<Angle>,
    /// Three axis of 'rotation speed' in rad/s
    pub angular_velocity: Vector3<AngularVelocity>,
    /// Three axis of magnetic field sensing in micro Tesla (uT)
    pub magnetic_field: Vector3<MagneticFluxDensity>,
}

#[derive(Debug)]
pub struct Vector3<T>
where
    T: Debug + Add + Div + Mul + Neg + Rem + Sub + AddAssign + RemAssign + SubAssign,
{
    x: T,
    y: T,
    z: T,
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
    x: Angle,
    z: Angle,
}

pub trait Avionics {
    fn guide(&mut self, sensors: &Sensors) -> Option<Control>;
}
