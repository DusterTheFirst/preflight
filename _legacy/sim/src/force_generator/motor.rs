use nalgebra::{Point3, Rotation2, Rotation3, Vector2, Vector3};
use nphysics3d::{
    algebra::Force3,
    force_generator::ForceGenerator,
    math::{Force, ForceType},
    object::BodyPartHandle,
    object::{BodyPart, BodySet, DefaultBodyPartHandle, DefaultColliderHandle},
    solver::IntegrationParameters,
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
    /// The angle in radians in the x and z direction
    angle: Rotation3<f64>,
}

impl<Table> RocketMotorForceGenerator<Table>
where
    Table: InterpolatedDataTable<Datapoint = RocketEngine, Time = f64>,
{
    pub fn new() -> Box<Self> {
        Box::new(Self {
            _table: PhantomData,
            parts: Vec::new(),
            total_time: 0.0,
            angle: Rotation3::from_euler_angles(0.0, 0.0, 0.0),
        })
    }

    pub fn add_body_part(&mut self, part: DefaultBodyPartHandle) -> &mut Self {
        self.parts.push(part);

        self
    }

    pub fn set_angle(&mut self, x: f64, z: f64) -> &mut Self {
        self.angle = Rotation3::from_euler_angles(0.0, x, z);

        self
    }

    pub fn thrust_vector(&self) -> Vector3<f64> {
        self.angle * (Vector3::y() * Table::get(self.total_time).thrust)
    }
}

impl<Table> ForceGenerator<f64, DefaultColliderHandle> for RocketMotorForceGenerator<Table>
where
    Table: InterpolatedDataTable<Datapoint = RocketEngine, Time = f64>,
{
    fn apply(
        &mut self,
        parameters: &IntegrationParameters<f64>,
        bodies: &mut dyn BodySet<f64, Handle = DefaultColliderHandle>,
    ) {
        for BodyPartHandle(body_handle, part_id) in self.parts.iter().cloned() {
            // Generate the force only if the body has not been removed from the world.
            if let Some(body) = bodies.get_mut(body_handle) {
                let part = body.part(part_id).unwrap();

                part.center_of_mass();

                let part_position = part.safe_position();

                // Transform the thrust vector so it is directly out the bottom of the part vs directly down
                let relative_thrust_vector = part_position
                    .rotation
                    .transform_vector(&self.thrust_vector());

                // TODO: VECTOR CONTROL

                // Apply the force.
                body.apply_local_force(
                    part_id,
                    &Force3::linear(relative_thrust_vector),
                    ForceType::Force,
                    false,
                );
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
