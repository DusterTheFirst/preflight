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
pub struct SupportedMotor {
    pub name: &'static str,
    pub thrust: fn(f64) -> MotorThrust,
    pub min: f64,
    pub max: f64,
}

impl PartialEq for SupportedMotor {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for SupportedMotor {}

impl Display for SupportedMotor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

pub const SUPPORTED_MOTORS: &[SupportedMotor] = &[
    SupportedMotor {
        name: "Estes A8",
        thrust: EstesA8::get,
        min: EstesA8::MIN,
        max: EstesA8::MAX,
    },
    SupportedMotor {
        name: "Estes B4",
        thrust: EstesB4::get,
        min: EstesB4::MIN,
        max: EstesB4::MAX,
    },
    SupportedMotor {
        name: "Estes C6",
        thrust: EstesC6::get,
        min: EstesC6::MIN,
        max: EstesC6::MAX,
    },
];
