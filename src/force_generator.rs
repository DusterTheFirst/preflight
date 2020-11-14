use timescale::{Lerp, InterpolatedData, InterpolatedDataTable};

#[derive(Lerp, InterpolatedData)]
pub struct RocketEngine {
    #[data(rename = "Thrust (N)")]
    thrust: f64,
}

#[derive(InterpolatedDataTable)]
#[table(file = "csv/Estes_C6.csv", st = RocketEngine)]
pub struct EstesC6;

#[derive(InterpolatedDataTable)]
#[table(file = "csv/Estes_B4.csv", st = RocketEngine)]
pub struct EstesB4;

#[derive(InterpolatedDataTable)]
#[table(file = "csv/Estes_A8.csv", st = RocketEngine)]
pub struct EstesA8;
