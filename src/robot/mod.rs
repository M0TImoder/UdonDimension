pub mod drive;

use bevy::math::primitives::Cuboid;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use drive::DriveInput;

pub struct RobotPlugin;

impl Plugin for RobotPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Startup, spawn_robot);
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
            DriveInput::default()
        )
    );
}
