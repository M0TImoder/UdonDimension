pub mod drive;

use bevy::math::primitives::Cuboid;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::core::time::{TickEvent, TimeSystem};
use drive::DriveInput;

pub struct RobotPlugin;

impl Plugin for RobotPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Startup, spawn_robot)
            .add_systems(Update, apply_drive_input_velocity.after(TimeSystem::Accumulate));
    }
}

fn spawn_robot(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
)
{
    commands.spawn(
        (
            PbrBundle
            {
                mesh: meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0)))),
                material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
                transform: Transform::from_xyz(0.0, 5.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(0.5, 0.5, 0.5),
            Velocity::default(),
            DriveInput::default()
        )
    );
}

fn apply_drive_input_velocity(
    mut tick_events: EventReader<TickEvent>,
    mut query: Query<(&DriveInput, &Transform, &mut Velocity), With<RigidBody>>
)
{
    for _event in tick_events.read()
    {
        for (drive_input, transform, mut velocity) in query.iter_mut()
        {
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
            linear_velocity.x = (forward * drive_input.vx.value()).x
                + (right * drive_input.vy.value()).x;
            linear_velocity.z = (forward * drive_input.vx.value()).z
                + (right * drive_input.vy.value()).z;

            velocity.linvel = linear_velocity;
            velocity.angvel = Vec3::new(0.0, drive_input.omega.value(), 0.0);
        }
    }
}
