use serde::Serialize;

pub trait Timescale {
    type Timescaled: Serialize;

    fn with_time(self, time: f64) -> Self::Timescaled;
}

#[allow(unused_imports)]
#[macro_use]
extern crate timescale_derive;

#[doc(hidden)]
pub use timescale_derive::*;
