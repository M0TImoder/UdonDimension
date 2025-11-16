use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};

pub const STANDARD_GRAVITY: f32 = 9.80665;
pub const SEA_LEVEL_AIR_DENSITY: f32 = 1.225;
pub const AIR_SCALE_HEIGHT: f32 = 8500.0;
pub const TURBULENCE_MAGNITUDE: f32 = 0.02;

#[derive(Resource, Clone)]
pub struct AirEnvironment
{
    pub sea_level_density: f32,
    pub scale_height: f32,
    pub base_wind: Vec3,
    pub turbulence_strength: f32,
    pub current_wind: Vec3,
    rng: SmallRng,
}

impl Default for AirEnvironment
{
    fn default() -> Self
    {
        Self
        {
            sea_level_density: SEA_LEVEL_AIR_DENSITY,
            scale_height: AIR_SCALE_HEIGHT,
            base_wind: Vec3::ZERO,
            turbulence_strength: TURBULENCE_MAGNITUDE,
            current_wind: Vec3::ZERO,
            rng: SmallRng::seed_from_u64(42),
        }
    }
}

impl AirEnvironment
{
    pub fn density_at_altitude(&self, altitude: f32) -> f32
    {
        let scaled = (-altitude.max(0.0) / self.scale_height).exp();
        self.sea_level_density * scaled
    }

    pub fn update_wind(&mut self)
    {
        let jitter = Vec3::new(
            self.rng.gen_range(-1.0..1.0),
            0.0,
            self.rng.gen_range(-1.0..1.0),
        ) * self.turbulence_strength;

        self.current_wind = self.base_wind + jitter;
    }
}

#[derive(Component, Clone, Copy)]
pub struct AirDrag
{
    pub coefficient: f32,
    pub angular_coefficient: f32,
    pub dimensions: Vec3,
}

impl AirDrag
{
    pub fn new(coefficient: f32, angular_coefficient: f32, dimensions: Vec3) -> Self
    {
        Self
        {
            coefficient,
            angular_coefficient,
            dimensions,
        }
    }

    pub fn projected_area(&self, direction: Vec3, rotation: &Quat) -> f32
    {
        if let Some(dir) = direction.try_normalize()
        {
            let axis_x = *rotation * Vec3::X;
            let axis_y = *rotation * Vec3::Y;
            let axis_z = *rotation * Vec3::Z;

            let area_x = self.dimensions.y * self.dimensions.z;
            let area_y = self.dimensions.x * self.dimensions.z;
            let area_z = self.dimensions.x * self.dimensions.y;

            let area_cap = area_x.max(area_y.max(area_z));

            return (area_x * dir.dot(axis_x).abs()
                + area_y * dir.dot(axis_y).abs()
                + area_z * dir.dot(axis_z).abs())
                .clamp(0.0, area_cap);
        }

        0.0
    }
}

pub fn update_air_environment(mut air: ResMut<AirEnvironment>)
{
    air.update_wind();
}

pub fn apply_aerodynamic_drag(
    air: Res<AirEnvironment>,
    mut query: Query<(&AirDrag, &Transform, &Velocity, &mut ExternalForce), With<RigidBody>>,
)
{
    for (drag, transform, velocity, mut force) in query.iter_mut()
    {
        let density = air.density_at_altitude(transform.translation.y);
        let relative_velocity = velocity.linvel - air.current_wind;
        let speed_sq = relative_velocity.length_squared();

        let mut drag_force = Vec3::ZERO;
        let mut drag_torque = Vec3::ZERO;

        if speed_sq > f32::EPSILON
        {
            let speed = speed_sq.sqrt();
            let area = drag.projected_area(relative_velocity, &transform.rotation);
            let drag_magnitude = 0.5 * density * speed_sq * drag.coefficient * area;
            let drag_direction = -relative_velocity / speed;

            drag_force = drag_direction * drag_magnitude;
        }

        if velocity.angvel.length_squared() > f32::EPSILON
        {
            drag_torque = -velocity.angvel * drag.angular_coefficient;
        }

        force.force = drag_force;
        force.torque = drag_torque;
    }
}
