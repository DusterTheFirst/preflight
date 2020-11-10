pub use lerp::Lerp;
use serde::Serialize;

/// Trait to allow data points to be expanded into their timescaled/serializable counterpart
/// by appending the time
pub trait ToTimescale {
    type Timescale: Serialize;

    /// Create the equivalent timescaled data point by appending the time
    /// to this data point
    fn with_time(self, time: f64) -> Self::Timescale;
}

pub enum TimescaleData<D> {
    Interpolation { low: D, high: D, percent: f64 },
    HighSaturation(D),
    LowSaturation(D),
}

/// Trait to allow for linear interpolation through a static timescale lookup table for smoother lookups
pub trait TimescaleDataTable {
    type Datapoint: Lerp<f64>;

    /// Get a data from the lookup table with all metadata attached. Prefer to use the `get_lerp` methods.
    fn get(time: f64) -> TimescaleData<Self::Datapoint>;

    /// Get data from the lookup table, linear interpolating between points
    /// if the time given is between 2 points on the table, fully saturating
    /// if the time is outside of the table's range
    fn get_lerp(time: f64) -> Self::Datapoint {
        match Self::get(time) {
            TimescaleData::HighSaturation(d) | TimescaleData::LowSaturation(d) => d,
            TimescaleData::Interpolation { low, high, percent } => low.lerp(high, percent),
        }
    }
}

#[allow(unused_imports)]
#[macro_use]
extern crate timescale_derive;

#[doc(hidden)]
pub use timescale_derive::*;

// #[cfg(test)]
mod test {
    use lerp::Lerp;

    use super::{TimescaleData, TimescaleDataTable};

    struct DataTable;
    impl TimescaleDataTable for DataTable {
        type Datapoint = f64;

        fn get(time: f64) -> TimescaleData<Self::Datapoint> {
            match time {
                _ if time <= 0.0 => TimescaleData::LowSaturation(0.0),
                // 0.0 => TimescaleData::Literal(0.0),
                _ if time >= 0.0 && time < 0.5 => {
                    let time_high = 0.5;
                    let time_low = 0.0;

                    TimescaleData::Interpolation {
                        high: 100.0,
                        low: 0.0,
                        percent: (time - time_low) / (time_high - time_low),
                    }
                }
                // 0.5 => TimescaleData::Literal(100.0),
                _ if time >= 0.5 && time < 1.0 => {
                    let time_high = 1.0;
                    let time_low = 0.5;

                    TimescaleData::Interpolation {
                        high: 300.0,
                        low: 100.0,
                        percent: (time - time_low) / (time_high - time_low),
                    }
                }
                // 1.0 => TimescaleData::Literal(300.0),
                _ if time >= 1.0 => TimescaleData::HighSaturation(300.0),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn datatable_1_unit() {
        assert_eq!(DataTable::get_lerp(-0.1), 0.0);
        // assert_eq!(DataTable::get_quantized(-0.1), None);

        assert_eq!(DataTable::get_lerp(0.0), 0.0);
        // assert_eq!(DataTable::get_quantized(0.0), Some(0.0));

        assert_eq!(DataTable::get_lerp(0.1), 20.0);
        // assert_eq!(DataTable::get_quantized(0.1), None);

        assert_eq!(DataTable::get_lerp(0.5), 100.0);
        // assert_eq!(DataTable::get_quantized(0.5), Some(100.0));

        assert_eq!(DataTable::get_lerp(0.6), 140.0);
        // assert_eq!(DataTable::get_quantized(0.6), None);

        assert_eq!(DataTable::get_lerp(1.0), 300.0);
        // assert_eq!(DataTable::get_quantized(1.0), Some(300.0));

        assert_eq!(DataTable::get_lerp(1.1), 300.0);
        // assert_eq!(DataTable::get_quantized(1.1), None);
    }

    struct DataTable2;
    struct Data2(f64, f64);

    impl TimescaleDataTable for DataTable2 {
        type Datapoint = Data2;

        fn get(time: f64) -> TimescaleData<Self::Datapoint> {
            match time {
                _ if time <= 0.0 => TimescaleData::LowSaturation(0.0),
                // 0.0 => TimescaleData::Literal(0.0),
                _ if time >= 0.0 && time < 0.5 => {
                    let time_high = 0.5;
                    let time_low = 0.0;

                    TimescaleData::Interpolation {
                        high: 100.0,
                        low: 0.0,
                        percent: (time - time_low) / (time_high - time_low),
                    }
                }
                // 0.5 => TimescaleData::Literal(100.0),
                _ if time >= 0.5 && time < 1.0 => {
                    let time_high = 1.0;
                    let time_low = 0.5;

                    TimescaleData::Interpolation {
                        high: 300.0,
                        low: 100.0,
                        percent: (time - time_low) / (time_high - time_low),
                    }
                }
                // 1.0 => TimescaleData::Literal(300.0),
                _ if time >= 1.0 => TimescaleData::HighSaturation(300.0),
                _ => unreachable!(),
            }
        }
    }


    #[test]
    fn datatable_2_units() {
        assert_eq!(DataTable::get_lerp(-0.1), 0.0);
        // assert_eq!(DataTable::get_quantized(-0.1), None);

        assert_eq!(DataTable::get_lerp(0.0), 0.0);
        // assert_eq!(DataTable::get_quantized(0.0), Some(0.0));

        assert_eq!(DataTable::get_lerp(0.1), 20.0);
        // assert_eq!(DataTable::get_quantized(0.1), None);

        assert_eq!(DataTable::get_lerp(0.5), 100.0);
        // assert_eq!(DataTable::get_quantized(0.5), Some(100.0));

        assert_eq!(DataTable::get_lerp(0.6), 140.0);
        // assert_eq!(DataTable::get_quantized(0.6), None);

        assert_eq!(DataTable::get_lerp(1.0), 300.0);
        // assert_eq!(DataTable::get_quantized(1.0), Some(300.0));

        assert_eq!(DataTable::get_lerp(1.1), 300.0);
        // assert_eq!(DataTable::get_quantized(1.1), None);
    }
}
