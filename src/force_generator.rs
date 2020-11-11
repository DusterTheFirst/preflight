use timescale::{Lerp, TimescaleDataTable};

#[derive(Lerp)]
pub struct RocketEngine {
    thrust: f64,
}

// load_csv!(from "../csv/Estes_C6.csv" load RocketEngine);

#[derive(TimescaleDataTable)]
#[csv(file = "csv/Estes_C6.csv", st = RocketEngine)]
pub struct EstesC6;
