use lerp::Lerp;
use timescale::{InterpolatedData, InterpolatedDataTable};

#[derive(Lerp, InterpolatedData)]
pub struct RocketEngine {
    #[data(rename = "Thrust (N)")]
    pub thrust: f64,
}

#[derive(InterpolatedDataTable)]
#[table(file = "motors/Estes_C6.csv", st = RocketEngine, j = "s")]
pub struct EstesC6;

fn main() {}
