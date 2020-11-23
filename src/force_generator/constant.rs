use nalgebra::{Point3, Vector3};
use nphysics3d::{
    force_generator::ForceGenerator,
    math::{Force, ForceType},
    object::{BodyPartHandle, BodySet, DefaultBodyPartHandle, DefaultColliderHandle},
    solver::IntegrationParameters,
};

pub struct Spin(Vec<DefaultBodyPartHandle>);

impl Spin {
    pub fn new(body_parts: &[DefaultBodyPartHandle]) -> Box<Self> {
        Box::new(Self(body_parts.to_vec()))
    }
}

impl ForceGenerator<f64, DefaultColliderHandle> for Spin {
    fn apply(
        &mut self,
        _: &IntegrationParameters<f64>,
        bodies: &mut dyn BodySet<f64, Handle = DefaultColliderHandle>,
    ) {
        for BodyPartHandle(body_handle, part_id) in self.0.iter().cloned() {
            if let Some(body) = bodies.get_mut(body_handle) {
                body.apply_force(
                    part_id,
                    &Force::torque_at_point(
                        Vector3::new(0.0, 30.0, 0.0),
                        &Point3::new(0.0, 10.0, 0.0),
                    ),
                    ForceType::VelocityChange,
                    false,
                );

                dbg!(part_id, Force::torque(Vector3::new(0.0, 10.0, 0.0)));
            }
        }
    }
}
