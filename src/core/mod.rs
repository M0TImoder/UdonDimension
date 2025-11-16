pub mod time;
pub mod units;

use bevy::app::AppExit;
use bevy::input::gamepad::{GamepadAxis, GamepadAxisType, Gamepads};
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseMotion;
use bevy::input::Axis;
use bevy::math::EulerRot;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowMode};
use crate::core::units::MetersPerSecond;

pub struct CorePlugin;

impl Plugin for CorePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, (camera_look, camera_move, handle_window_input));
    }
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

fn setup_camera(
    mut commands: Commands,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>
)
{
    if let Ok(mut window) = window_query.get_single_mut()
    {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

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
    axes: Res<Axis<GamepadAxis>>,
    gamepads: Res<Gamepads>,
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

    if let Some(gamepad) = gamepads.iter().next()
    {
        let axis_x = axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
            .unwrap_or(0.0);
        let axis_y = axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY))
            .unwrap_or(0.0);
        let stick_input = Vec2::new(axis_x, axis_y);
        let deadzone = 0.1;

        if stick_input.length_squared() > deadzone * deadzone
        {
            direction += forward * stick_input.y + right * stick_input.x;
        }
    }

    if direction.length_squared() == 0.0
    {
        return;
    }

    let direction = direction.normalize();
    let speed = MetersPerSecond::new(5.0);
    transform.translation += direction * speed.value() * time.delta_seconds();
}

fn camera_look(
    time: Res<Time>,
    axes: Res<Axis<GamepadAxis>>,
    gamepads: Res<Gamepads>,
    mut motion_events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>
)
{
    let mut delta = Vec2::ZERO;

    for event in motion_events.read()
    {
        delta += event.delta;
    }

    let (mut transform, mut controller) = match query.get_single_mut()
    {
        Ok(result) => result,
        Err(_) => return,
    };

    if let Some(gamepad) = gamepads.iter().next()
    {
        let axis_x = axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::RightStickX))
            .unwrap_or(0.0);
        let axis_y = axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::RightStickY))
            .unwrap_or(0.0);
        let stick_input = Vec2::new(axis_x, -axis_y);
        let deadzone = 0.1;

        if stick_input.length_squared() > deadzone * deadzone
        {
            let stick_speed = 2.5;
            delta += stick_input * stick_speed * time.delta_seconds() / controller.sensitivity;
        }
    }

    if delta == Vec2::ZERO
    {
        return;
    }

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

fn handle_window_input(
    input: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut exit_events: EventWriter<AppExit>
)
{
    if input.just_pressed(KeyCode::Escape)
    {
        exit_events.send(AppExit);
        return;
    }

    if !(input.just_pressed(KeyCode::Enter)
        && (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)))
    {
        return;
    }

    if let Ok(mut window) = window_query.get_single_mut()
    {
        let fullscreen = matches!(window.mode, WindowMode::BorderlessFullscreen | WindowMode::Fullscreen);

        window.mode = if fullscreen
        {
            WindowMode::Windowed
        }
        else
        {
            WindowMode::BorderlessFullscreen
        };
    }
}
