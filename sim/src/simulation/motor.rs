use std::fmt::Display;

use timescale::{InterpolatedData, InterpolatedDataTable, Lerp};

#[derive(Lerp, InterpolatedData, Copy, Clone)]
pub struct MotorThrust {
    #[data(rename = "Thrust (N)")]
    pub thrust: f64,
}

#[derive(InterpolatedDataTable)]
#[table(file = "../motors/Estes_C6.csv", st = MotorThrust)]
pub struct EstesC6;

#[derive(InterpolatedDataTable)]
#[table(file = "../motors/Estes_B4.csv", st = MotorThrust)]
pub struct EstesB4;

#[derive(InterpolatedDataTable)]
#[table(file = "../motors/Estes_A8.csv", st = MotorThrust)]
pub struct EstesA8;

// TODO: macro?
#[derive(Debug, Copy, Clone)]
pub struct RocketMotor {
    pub name: &'static str,
    pub thrust: fn(f64) -> MotorThrust,
    pub min: f64,
    pub max: f64,
}

impl PartialEq for RocketMotor {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for RocketMotor {}

impl Display for RocketMotor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

pub const SUPPORTED_MOTORS: &[RocketMotor] = &[
    RocketMotor {
        name: "Estes A8",
        thrust: EstesA8::get,
        min: EstesA8::MIN,
        max: EstesA8::MAX,
    },
    RocketMotor {
        name: "Estes B4",
        thrust: EstesB4::get,
        min: EstesB4::MIN,
        max: EstesB4::MAX,
    },
    RocketMotor {
        name: "Estes C6",
        thrust: EstesC6::get,
        min: EstesC6::MIN,
        max: EstesC6::MAX,
    },
];
