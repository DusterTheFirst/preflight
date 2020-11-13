use timescale::{Lerp, TimescaleData, TimescaleDataTable};

#[derive(Lerp, TimescaleData)]
pub struct RocketEngine {
    #[timescale(rename = "Thrust (N)")]
    thrust: f64,
}

#[derive(TimescaleDataTable)]
#[table(file = "csv/Estes_C6.csv", st = RocketEngine)]
pub struct EstesC6;

#[derive(Lerp, TimescaleData)]
pub struct H {
    #[timescale(rename = "Thrust (N)")]
    thrust: f32,
    #[timescale(rename = "H")]
    h: f32,
}

#[derive(TimescaleDataTable)]
#[table(file = "csv/H.csv", st = H)]
pub struct HHH;
