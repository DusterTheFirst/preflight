use lerp::Lerp;
use timescale::{InterpolatedData, InterpolatedDataTable};

#[derive(Lerp, InterpolatedData)]
pub struct RocketEngine {
    pub h: f64,
}

#[derive(InterpolatedDataTable)]
#[table(file = "../../../assets/motors/Estes_C6.csv", st = "RocketEngine")]
pub struct EstesC6;

fn main() {}
