pub use lerp::Lerp;
use serde::Serialize;

/// Trait to allow data points to be expanded into their timescaled and serializable counterpart
/// by appending the time
pub trait ToTimescale {
    type Timescale: Serialize;

    /// Create the equivalent timescaled data point by appending the time
    /// to this data point
    fn with_time(self, time: f64) -> Self::Timescale;
}

pub enum InterpolatedDataPoint<Table: InterpolatedDataTable> {
    Interpolation {
        prev: Table::Datapoint,
        next: Table::Datapoint,
        percent: Table::Time,
    },
    Saturation(Table::Datapoint),
}

/// Trait to allow for linear interpolation through a static timescale table
pub trait InterpolatedDataTable: Send + Sync + Sized + 'static {
    type Datapoint: Lerp<Self::Time>;
    type Time;

    /// Get a data from the lookup table with all metadata attached. Prefer to use the `get` method, for
    /// this method is an implementation detail
    fn get_raw(time: Self::Time) -> InterpolatedDataPoint<Self>;

    /// Get data from the lookup table, linear interpolating between points
    /// if the time given is between 2 points on the table, fully saturating
    /// if the time is outside of the table's range
    fn get(time: Self::Time) -> Self::Datapoint {
        match Self::get_raw(time) {
            InterpolatedDataPoint::Saturation(d) => d,
            InterpolatedDataPoint::Interpolation {
                prev,
                next,
                percent,
            } => prev.lerp(next, percent),
        }
    }
}

#[allow(unused_imports)]
#[macro_use]
extern crate timescale_macros;

#[doc(hidden)]
pub use timescale_macros::*;

#[cfg(test)]
mod test {
    use lerp::Lerp;

    use super::{InterpolatedDataPoint, InterpolatedDataTable};

    struct DataTable;
    impl InterpolatedDataTable for DataTable {
        type Datapoint = f32;
        type Time = f32;

        fn get_raw(time: Self::Time) -> InterpolatedDataPoint<Self> {
            match time {
                _ if time <= 0.0 => InterpolatedDataPoint::Saturation(0.0),
                // 0.0 => TimescaleData::Literal(0.0),
                _ if time >= 0.0 && time < 0.5 => {
                    let time_high = 0.5;
                    let time_low = 0.0;

                    InterpolatedDataPoint::Interpolation {
                        next: 100.0,
                        prev: 0.0,
                        percent: (time - time_low) / (time_high - time_low),
                    }
                }
                // 0.5 => TimescaleData::Literal(100.0),
                _ if time >= 0.5 && time < 1.0 => {
                    let time_high = 1.0;
                    let time_low = 0.5;

                    InterpolatedDataPoint::Interpolation {
                        next: 300.0,
                        prev: 100.0,
                        percent: (time - time_low) / (time_high - time_low),
                    }
                }
                // 1.0 => TimescaleData::Literal(300.0),
                _ if time >= 1.0 => InterpolatedDataPoint::Saturation(300.0),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn datatable_1_unit() {
        assert_eq!(DataTable::get(-0.1), 0.0);
        // assert_eq!(DataTable::get_quantized(-0.1), None);

        assert_eq!(DataTable::get(0.0), 0.0);
        // assert_eq!(DataTable::get_quantized(0.0), Some(0.0));

        assert_eq!(DataTable::get(0.1), 20.0);
        // assert_eq!(DataTable::get_quantized(0.1), None);

        assert_eq!(DataTable::get(0.5), 100.0);
        // assert_eq!(DataTable::get_quantized(0.5), Some(100.0));

        assert_eq!(DataTable::get(0.6), 140.0);
        // assert_eq!(DataTable::get_quantized(0.6), None);

        assert_eq!(DataTable::get(1.0), 300.0);
        // assert_eq!(DataTable::get_quantized(1.0), Some(300.0));

        assert_eq!(DataTable::get(1.1), 300.0);
        // assert_eq!(DataTable::get_quantized(1.1), None);
    }

    struct DataTable2;

    #[derive(Lerp, PartialEq, Debug)]
    struct Data2(f64, f64);

    // impl Lerp<f64> for Data2 {
    //     fn lerp(self, other: Self, t: f64) -> Self {
    //         Self(self.0.lerp(other.0, t), self.1.lerp(other.1, t))
    //     }
    // }

    impl InterpolatedDataTable for DataTable2 {
        type Datapoint = Data2;
        type Time = f64;

        fn get_raw(time: f64) -> InterpolatedDataPoint<Self> {
            match time {
                _ if time <= 0.0 => InterpolatedDataPoint::Saturation(Data2(0.0, 0.0)),
                // 0.0 => TimescaleData::Literal(0.0),
                _ if time >= 0.0 && time < 0.5 => {
                    let time_high = 0.5;
                    let time_low = 0.0;

                    InterpolatedDataPoint::Interpolation {
                        next: Data2(100.0, -100.0),
                        prev: Data2(0.0, 0.0),
                        percent: (time - time_low) / (time_high - time_low),
                    }
                }
                // 0.5 => TimescaleData::Literal(100.0),
                _ if time >= 0.5 && time < 1.0 => {
                    let time_high = 1.0;
                    let time_low = 0.5;

                    InterpolatedDataPoint::Interpolation {
                        next: Data2(300.0, -300.0),
                        prev: Data2(100.0, -100.0),
                        percent: (time - time_low) / (time_high - time_low),
                    }
                }
                // 1.0 => TimescaleData::Literal(300.0),
                _ if time >= 1.0 => InterpolatedDataPoint::Saturation(Data2(300.0, -300.0)),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn datatable_2_units() {
        assert_eq!(DataTable2::get(-0.1), Data2(0.0, 0.0));
        // assert_eq!(DataTable::get_quantized(-0.1), None);

        assert_eq!(DataTable2::get(0.0), Data2(0.0, 0.0));
        // assert_eq!(DataTable::get_quantized(0.0), Some(0.0));

        assert_eq!(DataTable2::get(0.1), Data2(20.0, -20.0));
        // assert_eq!(DataTable::get_quantized(0.1), None);

        assert_eq!(DataTable2::get(0.5), Data2(100.0, -100.0));
        // assert_eq!(DataTable::get_quantized(0.5), Some(100.0));

        assert_eq!(DataTable2::get(0.6), Data2(140.0, -140.0));
        // assert_eq!(DataTable::get_quantized(0.6), None);

        assert_eq!(DataTable2::get(1.0), Data2(300.0, -300.0));
        // assert_eq!(DataTable::get_quantized(1.0), Some(300.0));

        assert_eq!(DataTable2::get(1.1), Data2(300.0, -300.0));
        // assert_eq!(DataTable::get_quantized(1.1), None);
    }
}
