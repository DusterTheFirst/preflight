use timescale::{Lerp, TimescaleDataTable, TimescaleData};

#[derive(Lerp, TimescaleData)]
pub struct RocketEngine {
    #[rename("Thrust (N)")]
    thrust: f64,
}

#[derive(TimescaleDataTable)]
#[csv(file = "csv/Estes_C6.csv", st = RocketEngine)]
pub struct EstesC6;