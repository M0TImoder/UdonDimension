pub mod drive;

use crate::core::time::{TickEvent, TimeSystem};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use drive::DriveInput;

pub struct RobotPlugin;

impl Plugin for RobotPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Update, apply_drive_input_velocity.after(TimeSystem::Accumulate));
    }
}

fn apply_drive_input_velocity(
    mut tick_events: EventReader<TickEvent>,
    mut query: Query<(&DriveInput, &Transform, &mut Velocity), With<RigidBody>>,
)
{
    for _event in tick_events.read()
    {
        for (drive_input, transform, mut velocity) in query.iter_mut()
        {
            if transform.translation.is_nan() || velocity.linvel.is_nan() {
                continue;
            }

            let mut forward = Vec3::from(transform.forward());
            forward.y = 0.0;

            if let Some(normalized_forward) = forward.try_normalize()
            {
                forward = normalized_forward;
            }

            let mut right = Vec3::from(transform.right());
            right.y = 0.0;

            if let Some(normalized_right) = right.try_normalize()
            {
                right = normalized_right;
            }

            let mut linear_velocity = velocity.linvel;
            
            let new_x = (forward * drive_input.vx.value()).x + (right * drive_input.vy.value()).x;
            let new_z = (forward * drive_input.vx.value()).z + (right * drive_input.vy.value()).z;

            if !new_x.is_nan() {
                linear_velocity.x = new_x;
            }
            if !new_z.is_nan() {
                linear_velocity.z = new_z;
            }

            velocity.linvel = linear_velocity;
            
            if !drive_input.omega.value().is_nan() {
                velocity.angvel = Vec3::new(0.0, drive_input.omega.value(), 0.0);
            }
        }
    }
}
