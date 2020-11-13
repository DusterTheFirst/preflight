use timescale::{Lerp, TimescaleData, TimescaleDataTable};

#[derive(Lerp, TimescaleData)]
pub struct RocketEngine {
    #[timescale(rename = "Thrust (N)")]
    thrust: f64,
}

#[derive(TimescaleDataTable)]
#[table(file = "csv/Estes_C6.csv", st = RocketEngine)]
pub struct EstesC6;

#[derive(TimescaleDataTable)]
#[table(file = "csv/Estes_B4.csv", st = RocketEngine)]
pub struct EstesB4;

#[derive(TimescaleDataTable)]
#[table(file = "csv/Estes_A8.csv", st = RocketEngine)]
pub struct EstesA8;
