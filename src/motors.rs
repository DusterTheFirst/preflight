use std::marker::PhantomData;

use nalgebra::{Point3, Vector3};
use nphysics3d::{
    force_generator::ForceGenerator,
    math::{Force, ForceType},
    object::{BodyPartHandle, DefaultBodyPartHandle, DefaultColliderHandle, RigidBody},
};
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
