use lerp::Lerp;
use timescale::{InterpolatedData, InterpolatedDataTable};

#[derive(Lerp, InterpolatedData)]
pub struct RocketEngine {
    #[data(rename = "Thrust (N)")]
    pub thrust: f64,
}

#[derive(InterpolatedDataTable)]
#[table(file = "../../../assets/motors/Estes_A8.csv", st = "RocketEngine")]
pub struct EstesA8;

fn main() {}
