#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

pub use preflight_macros::avionics_harness;
pub use uom;
use uom::si::{
    acceleration, angle, angular_velocity, length, magnetic_flux_density, Dimension, SI,
};

pub type Quantity<T> = uom::si::Quantity<T, SI<f64>, f64>;

#[repr(C)]
#[derive(Debug)]
pub struct Sensors {
    /// Calculated altitude
    pub altitude: Quantity<length::Dimension>,
    /// Three axis of linear acceleration data (acceleration minus gravity) in m/s^2
    pub linear_acceleration: Vector3<acceleration::Dimension>,
    /// Three axis of gravitational acceleration (minus any movement) in m/s^2
    pub gravity_acceleration: Vector3<acceleration::Dimension>,
    /// Three axis of acceleration (gravity + linear motion) in m/s^2
    pub both_acceleration: Vector3<acceleration::Dimension>,
    /// Three axis orientation data based on a 360° sphere
    pub orientation: Vector3<angle::Dimension>,
    /// Three axis of 'rotation speed' in rad/s
    pub angular_velocity: Vector3<angular_velocity::Dimension>,
    /// Three axis of magnetic field sensing in micro Tesla (uT)
    pub magnetic_field: Vector3<magnetic_flux_density::Dimension>,
}

#[repr(C)]
pub struct Vector3<T: Dimension + ?Sized> {
    x: Quantity<T>,
    y: Quantity<T>,
    z: Quantity<T>,
}

impl<T: Dimension + ?Sized> Debug for Vector3<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // if f.alternate() {

        // }
        // write!(f, "({}, {}, {})", self.x.value, self.y.value, self.z);
        f.debug_struct("Vector3")
            .field("x", &self.x) //Quantity::format_args(self.x, Abbreviation)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish()
    }
}

impl<T: Dimension + ?Sized> Vector3<T> {
    pub fn new(x: Quantity<T>, y: Quantity<T>, z: Quantity<T>) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: Quantity {
                dimension: PhantomData,
                units: PhantomData,
                value: 0.0,
            },
            y: Quantity {
                dimension: PhantomData,
                units: PhantomData,
                value: 0.0,
            },
            z: Quantity {
                dimension: PhantomData,
                units: PhantomData,
                value: 0.0,
            },
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub enum Control {
    ABORT(AbortCause),
    Guidance(Guidance),
}

#[repr(C)]
#[derive(Debug)]
pub enum AbortCause {
    ControlFailure = 0,
}

#[repr(C)]
#[derive(Debug)]
pub struct Guidance {
    pub tvc: ThrustVector,
    // TODO: pyro
}

#[repr(C)]
#[derive(Debug)]
pub struct ThrustVector {
    x: Quantity<angle::Dimension>,
    z: Quantity<angle::Dimension>,
}

pub trait Avionics: Debug + Send + Sync {
    fn guide(&mut self, sensors: &Sensors) -> Option<Control>;
}
