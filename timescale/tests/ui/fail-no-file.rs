use lerp::Lerp;
use timescale::{InterpolatedData, InterpolatedDataTable};

#[derive(Lerp, InterpolatedData)]
pub struct RocketEngine {
    #[data(rename = "Thrust (N)")]
    pub thrust: f64,
}

#[derive(InterpolatedDataTable)]
#[table(file = "../../../assets/motors/ghajkhgdaskjdgh.csv", st = "RocketEngine")]
pub struct EstesC6;

fn main() {}