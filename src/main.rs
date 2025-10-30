use bevy::math::primitives::{Cuboid, Plane3d};
use bevy::prelude::*;

fn main()
{
    App::new()
        .add_plugins(
            DefaultPlugins.set(
                WindowPlugin
                {
                    primary_window: Some(
                        Window
                        {
                            title: "UdonDimension Prototype".into(),
                            ..default()
                        }
                    ),
                    ..default()
                }
            )
        )
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
)
{
    commands.spawn(
        PbrBundle
        {
            mesh: meshes.add(Mesh::from(Plane3d::default())),
            material: materials.add(Color::rgb(0.5, 0.7, 0.5)),
            transform: Transform::from_scale(Vec3::new(20.0, 1.0, 20.0)),
            ..default()
        }
    );

    commands.spawn(
        PbrBundle
        {
            mesh: meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0)))),
            material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        }
    );

    commands.spawn(
        DirectionalLightBundle
        {
            directional_light: DirectionalLight
            {
                shadows_enabled: true,
                illuminance: 2000.0,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        }
    );

    commands.spawn(
        Camera3dBundle
        {
            transform: Transform::from_xyz(-6.0, 6.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        }
    );
}
