pub mod drive;

use crate::core::time::{TickEvent, TimeSystem};
use crate::physics::drag::AirDrag;
use bevy::math::primitives::Cuboid;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
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
    mut materials: ResMut<Assets<StandardMaterial>>,
)
{
    let box_size = Vec3::splat(1.0);
    let half_extents = box_size * 0.5;
    let mass = 20.0;
    let inertia = Vec3
    {
        x: (mass / 12.0) * (box_size.y * box_size.y + box_size.z * box_size.z),
        y: (mass / 12.0) * (box_size.x * box_size.x + box_size.z * box_size.z),
        z: (mass / 12.0) * (box_size.x * box_size.x + box_size.y * box_size.y),
    };

    commands.spawn(
        (
            PbrBundle
            {
                mesh: meshes.add(Mesh::from(Cuboid::from_size(box_size))),
                material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
                transform: Transform::from_xyz(0.0, 5.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Ccd::enabled(),
            Collider::cuboid(half_extents.x, half_extents.y, half_extents.z),
            Friction
            {
                coefficient: 1.5,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution
            {
                coefficient: 0.1,
                combine_rule: CoefficientCombineRule::Min,
            },
            Velocity::default(),
            ExternalForce::default(),
            AdditionalMassProperties::MassProperties(
                MassProperties
                {
                    local_center_of_mass: Vec3::new(0.0, -half_extents.y * 0.15, 0.0),
                    mass,
                    principal_inertia_local_frame: Quat::IDENTITY,
                    principal_inertia: inertia,
                }
            ),
            AirDrag::new(1.05, 0.05, Vec3::splat(1.0)),
            DriveInput::default(),
        )
    );
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
