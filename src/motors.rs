use nalgebra::Vector3;
use nphysics3d::{
    force_generator::ForceGenerator,
    math::{Force, ForceType},
    object::{DefaultBodyPartHandle, DefaultColliderHandle},
};
use std::marker::PhantomData;
use timescale::{InterpolatedData, InterpolatedDataTable, Lerp};

#[derive(Lerp, InterpolatedData)]
pub struct RocketEngine {
    #[data(rename = "Thrust (N)")]
    pub thrust: f64,
}

pub struct RocketMotorForceGenerator<Table>
where
    Table: InterpolatedDataTable<Datapoint = RocketEngine, Time = f64>,
{
    _table: PhantomData<Table>,
    /// Body parts affected by the force generator
    parts: Vec<DefaultBodyPartHandle>,
    total_time: f64,
}

impl<Table> RocketMotorForceGenerator<Table>
where
    Table: InterpolatedDataTable<Datapoint = RocketEngine, Time = f64>,
{
    pub fn new() -> Self {
        Self {
            _table: PhantomData,
            parts: Vec::new(),
            total_time: 0.0,
        }
    }

    pub fn add_body_part(&mut self, part: DefaultBodyPartHandle) -> &mut Self {
        self.parts.push(part);

        self
    }
}

impl<Table> ForceGenerator<f64, DefaultColliderHandle> for RocketMotorForceGenerator<Table>
where
    Table: InterpolatedDataTable<Datapoint = RocketEngine, Time = f64>,
{
    fn apply(
        &mut self,
        parameters: &nphysics3d::solver::IntegrationParameters<f64>,
        bodies: &mut dyn nphysics3d::object::BodySet<f64, Handle = DefaultColliderHandle>,
    ) {
        for handle in &self.parts {
            // Generate the force only if the body has not been removed from the world.
            if let Some(body) = bodies.get_mut(handle.0) {
                let part = body.part(handle.1).unwrap();

                // TODO: VECTOR CONTROL
                let force = Force::linear(Vector3::y() * Table::get(self.total_time).thrust);

                // Apply the force.
                body.apply_force(handle.1, &force, ForceType::Force, false);
            }
        }

        self.total_time += parameters.dt();
    }
}

#[derive(InterpolatedDataTable)]
#[table(file = "motors/Estes_C6.csv", st = RocketEngine)]
pub struct EstesC6;

#[derive(InterpolatedDataTable)]
#[table(file = "motors/Estes_B4.csv", st = RocketEngine)]
pub struct EstesB4;

#[derive(InterpolatedDataTable)]
#[table(file = "motors/Estes_A8.csv", st = RocketEngine)]
pub struct EstesA8;
