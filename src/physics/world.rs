use bevy::math::primitives::Plane3d;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
)
{
    commands
        .spawn(
            (
                PbrBundle
                {
                    mesh: meshes.add(Mesh::from(Plane3d::default())),
                    material: materials.add(Color::rgb(0.5, 0.7, 0.5)),
                    transform: Transform::from_scale(Vec3::new(20.0, 1.0, 20.0)),
                    ..default()
                },
                RigidBody::Fixed
            )
        )
        .with_children(|parent|
        {
            parent.spawn(
                (
                    TransformBundle::from(
                        Transform::from_translation(Vec3::new(0.0, -0.5, 0.0))
                    ),
                    Collider::cuboid(10.0, 0.5, 10.0)
                )
            );
        });
}
