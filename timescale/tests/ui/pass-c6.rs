use lerp::Lerp;
use timescale::{InterpolatedData, InterpolatedDataTable};

#[derive(Lerp, InterpolatedData)]
pub struct RocketEngine {
    #[data(rename = "Thrust (N)")]
    pub thrust: f64,
}

// FIXME: Suboptimal to have to use strings for all, look into https://github.com/TedDriggs/darling/issues/108
#[derive(InterpolatedDataTable)]
#[table(file = "../../../assets/motors/Estes_C6.csv", st = "RocketEngine")]
pub struct EstesC6;

fn main() {}
