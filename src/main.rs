use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseMotion;
use bevy::math::{primitives::{Cuboid, Plane3d}, EulerRot};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;

fn main()
{
    App::new()
        .add_plugins(
            (
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
                ),
                RapierPhysicsPlugin::<NoUserData>::default()
            )
        )
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_look, camera_move))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>
)
{
    if let Ok(mut window) = window_query.get_single_mut()
    {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    commands.spawn(
        (
            PbrBundle
            {
                mesh: meshes.add(Mesh::from(Plane3d::default())),
                material: materials.add(Color::rgb(0.5, 0.7, 0.5)),
                transform: Transform::from_scale(Vec3::new(20.0, 1.0, 20.0)),
                ..default()
            },
            RigidBody::Fixed,
            Collider::cuboid(10.0, 0.5, 10.0)
        )
    );

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
            Collider::cuboid(0.5, 0.5, 0.5)
        )
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

    let camera_transform = Transform::from_xyz(-6.0, 6.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y);
    let controller = CameraController::from_transform(&camera_transform);

    commands.spawn(
        (
            Camera3dBundle
            {
                transform: camera_transform,
                ..default()
            },
            controller
        )
    );
}

fn camera_move(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>
)
{
    let mut transform = match query.get_single_mut()
    {
        Ok(transform) => transform,
        Err(_) => return,
    };

    let forward = Vec3::from(transform.forward());
    let mut direction = Vec3::ZERO;

    if input.pressed(KeyCode::KeyW)
    {
        direction += forward;
    }

    if input.pressed(KeyCode::KeyS)
    {
        direction -= forward;
    }

    let mut right = Vec3::from(transform.right());
    right.y = 0.0;

    if input.pressed(KeyCode::KeyA)
    {
        direction -= right;
    }

    if input.pressed(KeyCode::KeyD)
    {
        direction += right;
    }

    if direction.length_squared() == 0.0
    {
        return;
    }

    let direction = direction.normalize();
    let speed = 5.0;
    transform.translation += direction * speed * time.delta_seconds();
}

fn camera_look(
    mut motion_events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>
)
{
    let mut delta = Vec2::ZERO;

    for event in motion_events.read()
    {
        delta += event.delta;
    }

    if delta == Vec2::ZERO
    {
        return;
    }

    let (mut transform, mut controller) = match query.get_single_mut()
    {
        Ok(result) => result,
        Err(_) => return,
    };

    controller.yaw -= delta.x * controller.sensitivity;
    controller.pitch -= delta.y * controller.sensitivity;
    // ピッチ反転防止閾値
    controller.pitch = controller
        .pitch
        .clamp(controller.pitch_limits.x, controller.pitch_limits.y);

    let yaw_rotation = Quat::from_rotation_y(controller.yaw);
    let pitch_rotation = Quat::from_rotation_x(controller.pitch);
    transform.rotation = yaw_rotation * pitch_rotation;
}

#[derive(Component)]
struct CameraController
{
    yaw: f32,
    pitch: f32,
    pitch_limits: Vec2,
    sensitivity: f32,
}

impl CameraController
{
    fn from_transform(transform: &Transform) -> Self
    {
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

        Self
        {
            yaw,
            pitch,
            pitch_limits: Vec2::new(-1.54, 1.54),
            sensitivity: 0.0025,
        }
    }
}
