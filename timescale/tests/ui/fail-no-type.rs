use lerp::Lerp;
use timescale::InterpolatedDataTable;

#[derive(InterpolatedDataTable)]
#[table(file = "motors/Estes_C6.csv", st = NotExist)]
pub struct EstesC6;

fn main() {}
