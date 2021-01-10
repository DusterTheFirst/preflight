//! Utilities to create and test hardware agnostic flight systems with little friction
//!
//! This preflight crate goes hand and hand with the [`cargo_preflight`] cargo
//! subcommand. Alone, this crate will produce a stable, plugable interface for
//! a flight computer to link to. When paired with the [`cargo_preflight`] command,
//! users are able to run the flight systems through rigorous simulations and
//! tests to verify their integrity.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

pub use preflight_macros::avionics_harness;
pub use uom;
use uom::si::{
    acceleration, angle, angular_velocity, length, magnetic_flux_density, Dimension, SI,
};

pub mod abi;

/// Generic [`uom`] quantity using f64 as the storage type
pub type Quantity<T> = uom::si::Quantity<T, SI<f64>, f64>;

/// Generic sensor values that are collected from the flight hardware which would
/// be useful to calculate position and velocity.
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

/// A vector representing a quantity in 3 dimensional space
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
    /// Create a 3 dimensional vector from 3 quantities
    pub fn new(x: Quantity<T>, y: Quantity<T>, z: Quantity<T>) -> Self {
        Self { x, y, z }
    }

    /// Create a zeroed vector
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

/// Various control signals that the avionics can produce
#[repr(C)]
#[derive(Debug)]
pub enum Control {
    /// An abort signal
    ///
    /// This will stop the system from calling the guidance avionics, and attempt
    /// to enter the vehicle into a recovery or failsafe mode
    ABORT(AbortCause),
    /// A firmware agnostic guidance control signal for the underlying firmware
    /// to translate into flight hardware specific servo movements of pyro channel
    /// fires
    Guidance(Guidance),
    /// A signal to the underlying flight system that the avionics was unable to
    /// compute a valid control signal for the time. The flight system will normally
    /// choose to request another guidance control immediately.
    RecoverableFailure,
}

/// Underlying cause for an abort
///
/// This cause can signal to the abort handler what recovery mode or actions need
/// to be taken and the severity of the abort
#[repr(C)]
#[derive(Debug)]
pub enum AbortCause {
    /// TODO: No abort causes exist currently
    TODO,
}

/// A hardware agnostic guidance signal
#[repr(C)]
#[derive(Debug)]
pub struct Guidance {
    /// Thrust vectoring control
    pub tvc: ThrustVector,
    // TODO: pyro/flags
}

/// A call for thrust vectoring hardware to produce a thrust at the given
/// vector
#[repr(C)]
#[derive(Debug)]
pub struct ThrustVector {
    /// The thrust on the x axis
    pub x: Quantity<angle::Dimension>,
    /// The thrust on the z axis
    pub z: Quantity<angle::Dimension>,
}

/// Hardware agnostic avionics system
///
/// Implementations of this trait should have the [`avionics_harness`] attribute
/// macro preceding them. Alone, this trait has little use.
pub trait Avionics: Debug + Send + Sync {
    /// Produce a control signal given the current sensor values
    ///
    /// This function can be thought of as the control signal generation step
    /// in a control loop
    fn guide(&mut self, sensors: &Sensors) -> Option<Control>;
    // TODO: ABORT HANDLE
    // TODO: CUSTOM SENSORS OR CONTROL STRUCT/ENUM?
}
