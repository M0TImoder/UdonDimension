use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseMotion;
use bevy::math::{
    primitives::{Cuboid, Plane3d},
    EulerRot,
};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use sdl2::controller::{
    Axis as GameControllerAxis, Button as GameControllerButton, GameController,
};
use sdl2::event::Event;
use sdl2::GameControllerSubsystem;
use std::collections::HashMap;
use std::fmt::Write;

fn main()
{
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "UdonDimension Prototype".into(),
                    ..default()
                }),
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (camera_look, camera_move, process_controller_events),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
)
{
    if let Ok(mut window) = window_query.get_single_mut() {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Plane3d::default())),
            material: materials.add(Color::rgb(0.5, 0.7, 0.5)),
            transform: Transform::from_scale(Vec3::new(20.0, 1.0, 20.0)),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.5, 10.0),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0)))),
            material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
            transform: Transform::from_xyz(0.0, 5.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 0.5),
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 2000.0,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let camera_transform = Transform::from_xyz(-6.0, 6.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y);
    let controller = CameraController::from_transform(&camera_transform);

    commands.spawn((
        Camera3dBundle {
            transform: camera_transform,
            ..default()
        },
        controller,
    ));

    commands.add(|world: &mut World| {
        world.insert_non_send_resource(SdlControllerManager::new());
    });

    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "No controllers connected",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                left: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        ControllerDisplay,
    ));
}

fn camera_move(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
)
{
    let mut transform = match query.get_single_mut() {
        Ok(transform) => transform,
        Err(_) => return,
    };

    let forward = Vec3::from(transform.forward());
    let mut direction = Vec3::ZERO;

    if input.pressed(KeyCode::KeyW) {
        direction += forward;
    }

    if input.pressed(KeyCode::KeyS) {
        direction -= forward;
    }

    let mut right = Vec3::from(transform.right());
    right.y = 0.0;

    if input.pressed(KeyCode::KeyA) {
        direction -= right;
    }

    if input.pressed(KeyCode::KeyD) {
        direction += right;
    }

    if direction.length_squared() == 0.0 {
        return;
    }

    let direction = direction.normalize();
    let speed = 5.0;
    transform.translation += direction * speed * time.delta_seconds();
}

fn camera_look(
    mut motion_events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
)
{
    let mut delta = Vec2::ZERO;

    for event in motion_events.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let (mut transform, mut controller) = match query.get_single_mut() {
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

#[derive(Component)]
struct ControllerDisplay;

struct ControllerAxes
{
    left: Vec2,
    right: Vec2,
    trigger_left: f32,
    trigger_right: f32,
}

impl Default for ControllerAxes
{
    fn default() -> Self
    {
        Self {
            left: Vec2::ZERO,
            right: Vec2::ZERO,
            trigger_left: 0.0,
            trigger_right: 0.0,
        }
    }
}

struct ControllerEntry
{
    index: usize,
    #[allow(dead_code)]
    controller: GameController,
    buttons: Vec<GameControllerButton>,
    axes: ControllerAxes,
}

impl ControllerEntry
{
    fn new(index: usize, controller: GameController) -> Self
    {
        Self {
            index,
            controller,
            buttons: Vec::new(),
            axes: ControllerAxes::default(),
        }
    }
}

struct SdlControllerManager
{
    sdl: sdl2::Sdl,
    controller_subsystem: GameControllerSubsystem,
    controllers: HashMap<u32, ControllerEntry>,
    next_index: usize,
}

impl SdlControllerManager
{
    fn new() -> Self
    {
        let sdl = sdl2::init().expect("SDL2 initialization failed");
        let controller_subsystem = sdl
            .game_controller()
            .expect("Controller subsystem init failed");

        let mut manager = Self {
            sdl,
            controller_subsystem,
            controllers: HashMap::new(),
            next_index: 0,
        };

        manager.scan_existing_controllers();

        manager
    }

    fn scan_existing_controllers(&mut self)
    {
        let Ok(count) = self.controller_subsystem.num_joysticks() else {
            return;
        };

        for device_index in 0..count {
            self.open_controller(device_index as u32);
        }
    }

    fn poll_events(&mut self) -> Vec<Event>
    {
        let mut collected = Vec::new();

        if let Ok(mut pump) = self.sdl.event_pump() {
            for event in pump.poll_iter() {
                collected.push(event);
            }
        }

        collected
    }

    fn open_controller(&mut self, device_index: u32)
    {
        if !self.controller_subsystem.is_game_controller(device_index) {
            return;
        }

        let Ok(controller) = self.controller_subsystem.open(device_index) else {
            return;
        };

        let instance_id = controller.instance_id();

        if self.controllers.contains_key(&instance_id) {
            return;
        }

        let entry = ControllerEntry::new(self.next_index, controller);
        self.next_index += 1;
        self.controllers.insert(instance_id, entry);
    }

    fn remove_controller(&mut self, instance_id: u32)
    {
        self.controllers.remove(&instance_id);
    }

    fn update_button(&mut self, instance_id: u32, button: GameControllerButton, pressed: bool)
    {
        if let Some(entry) = self.controllers.get_mut(&instance_id) {
            if pressed {
                if !entry.buttons.iter().any(|stored| *stored == button) {
                    entry.buttons.push(button);
                    entry.buttons.sort_by_key(|stored| *stored as i32);
                }
            } else if let Some(position) = entry.buttons.iter().position(|stored| *stored == button)
            {
                entry.buttons.remove(position);
            }
        }
    }

    fn update_axis(&mut self, instance_id: u32, axis: GameControllerAxis, value: i16)
    {
        if let Some(entry) = self.controllers.get_mut(&instance_id) {
            let numeric = value as f32;

            match axis {
                GameControllerAxis::LeftX => entry.axes.left.x = numeric,
                GameControllerAxis::LeftY => entry.axes.left.y = numeric,
                GameControllerAxis::RightX => entry.axes.right.x = numeric,
                GameControllerAxis::RightY => entry.axes.right.y = numeric,
                GameControllerAxis::TriggerLeft => entry.axes.trigger_left = numeric,
                GameControllerAxis::TriggerRight => entry.axes.trigger_right = numeric,
            }
        }
    }

    fn handle_event(&mut self, event: Event)
    {
        match event {
            Event::ControllerDeviceAdded { which, .. } => {
                self.open_controller(which);
            }
            Event::ControllerDeviceRemoved { which, .. } => self.remove_controller(which),
            Event::ControllerButtonDown { which, button, .. } => {
                self.update_button(which, button, true)
            }
            Event::ControllerButtonUp { which, button, .. } => {
                self.update_button(which, button, false)
            }
            Event::ControllerAxisMotion {
                which, axis, value, ..
            } => self.update_axis(which, axis, value),
            _ => {}
        }
    }

    fn compose_display(&self) -> String
    {
        if self.controllers.is_empty() {
            return "No controllers connected".to_string();
        }

        let mut entries: Vec<&ControllerEntry> = self.controllers.values().collect();
        entries.sort_by_key(|entry| entry.index);

        let mut output = String::new();

        for (idx, entry) in entries.iter().enumerate() {
            if idx > 0 {
                output.push('\n');
            }

            let _ = write!(output, "Controller{}: ", entry.index);

            if entry.buttons.is_empty() {
                output.push_str("Buttons[] ");
            } else {
                output.push_str("Buttons[");

                for (button_idx, button) in entry.buttons.iter().enumerate() {
                    if button_idx > 0 {
                        output.push(' ');
                    }

                    output.push_str(button_label(*button));
                }

                output.push_str("] ");
            }

            let _ = write!(
                output,
                "LStick: {:.2},{:.2} RStick: {:.2},{:.2} LTrigger: {:.2} RTrigger: {:.2}",
                entry.axes.left.x,
                entry.axes.left.y,
                entry.axes.right.x,
                entry.axes.right.y,
                entry.axes.trigger_left,
                entry.axes.trigger_right
            );
        }

        output
    }
}

fn button_label(button: GameControllerButton) -> &'static str
{
    match button {
        GameControllerButton::A => "A",
        GameControllerButton::B => "B",
        GameControllerButton::X => "X",
        GameControllerButton::Y => "Y",
        GameControllerButton::Back => "Back",
        GameControllerButton::Guide => "Guide",
        GameControllerButton::Start => "Start",
        GameControllerButton::LeftStick => "LStick",
        GameControllerButton::RightStick => "RStick",
        GameControllerButton::LeftShoulder => "L",
        GameControllerButton::RightShoulder => "R",
        GameControllerButton::DPadUp => "DPadUp",
        GameControllerButton::DPadDown => "DPadDown",
        GameControllerButton::DPadLeft => "DPadLeft",
        GameControllerButton::DPadRight => "DPadRight",
        GameControllerButton::Misc1 => "Misc1",
        GameControllerButton::Paddle1 => "Paddle1",
        GameControllerButton::Paddle2 => "Paddle2",
        GameControllerButton::Paddle3 => "Paddle3",
        GameControllerButton::Paddle4 => "Paddle4",
        GameControllerButton::Touchpad => "Touchpad",
    }
}

fn process_controller_events(
    mut manager: NonSendMut<SdlControllerManager>,
    mut display: Query<&mut Text, With<ControllerDisplay>>,
)
{
    let events = manager.poll_events();

    for event in events {
        manager.handle_event(event);
    }

    if let Ok(mut text) = display.get_single_mut() {
        text.sections[0].value = manager.compose_display();
    }
}

impl CameraController
{
    fn from_transform(transform: &Transform) -> Self
    {
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

        Self {
            yaw,
            pitch,
            pitch_limits: Vec2::new(-1.54, 1.54),
            sensitivity: 0.0025,
        }
    }
}
