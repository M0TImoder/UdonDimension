use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseMotion;
use bevy::math::{
    primitives::{Cuboid, Plane3d},
    EulerRot,
};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;

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

    #[cfg(not(target_os = "windows"))]
    commands.insert_resource(GamepadManager::new());

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

#[cfg(target_os = "windows")]
mod gamepad
{
    use bevy::math::Vec2;
    use gilrs::{
        Axis as GilrsAxis,
        Button as GilrsButton,
        Event as GilrsEvent,
        EventType as GilrsEventType,
        GamepadId,
        Gilrs,
    };
    use std::collections::HashMap;
    use std::fmt::Write;

    #[derive(Default)]
    struct ControllerAxes
    {
        left: Vec2,
        right: Vec2,
        trigger_left: f32,
        trigger_right: f32,
    }

    struct ControllerEntry
    {
        index: usize,
        buttons: Vec<GilrsButton>,
        axes: ControllerAxes,
    }

    impl ControllerEntry
    {
        fn new(index: usize) -> Self
        {
            Self {
                index,
                buttons: Vec::new(),
                axes: ControllerAxes::default(),
            }
        }
    }

    pub struct GamepadManager
    {
        gilrs: Gilrs,
        controllers: HashMap<GamepadId, ControllerEntry>,
        next_index: usize,
    }

    impl GamepadManager
    {
        pub fn new() -> Self
        {
            let mut gilrs = Gilrs::new().expect("Gilrs initialization failed");

            let mut manager = Self {
                gilrs,
                controllers: HashMap::new(),
                next_index: 0,
            };

            manager.scan_existing_controllers();

            manager
        }

        fn scan_existing_controllers(&mut self)
        {
            let ids: Vec<GamepadId> = self
                .gilrs
                .gamepads()
                .map(|(id, _)| id)
                .collect();

            for id in ids
            {
                self.add_controller(id);
                self.refresh_state(id);
            }
        }

        pub fn poll_events(&mut self)
        {
            while let Some(event) = self.gilrs.next_event()
            {
                self.handle_event(event);
            }

            self.gilrs.inc();
        }

        fn add_controller(&mut self, id: GamepadId)
        {
            if self.controllers.contains_key(&id)
            {
                return;
            }

            let entry = ControllerEntry::new(self.next_index);
            self.next_index += 1;
            self.controllers.insert(id, entry);
        }

        fn remove_controller(&mut self, id: GamepadId)
        {
            self.controllers.remove(&id);
        }

        fn update_button(&mut self, id: GamepadId, button: GilrsButton, pressed: bool)
        {
            if let Some(entry) = self.controllers.get_mut(&id)
            {
                if pressed
                {
                    if !entry.buttons.iter().any(|stored| *stored == button)
                    {
                        entry.buttons.push(button);
                        entry.buttons.sort_by_key(|stored| *stored as u16);
                    }
                }
                else if let Some(position) = entry.buttons.iter().position(|stored| *stored == button)
                {
                    entry.buttons.remove(position);
                }
            }
        }

        fn update_axis(&mut self, id: GamepadId, axis: GilrsAxis, value: f32)
        {
            if let Some(entry) = self.controllers.get_mut(&id)
            {
                match axis
                {
                    GilrsAxis::LeftStickX => entry.axes.left.x = value,
                    GilrsAxis::LeftStickY => entry.axes.left.y = value,
                    GilrsAxis::RightStickX => entry.axes.right.x = value,
                    GilrsAxis::RightStickY => entry.axes.right.y = value,
                    GilrsAxis::LeftZ => entry.axes.trigger_left = value,
                    GilrsAxis::RightZ => entry.axes.trigger_right = value,
                    _ => {}
                }
            }
        }

        fn refresh_state(&mut self, id: GamepadId)
        {
            if let Some(entry) = self.controllers.get_mut(&id)
            {
                if let Some(gamepad) = self.gilrs.connected_gamepad(id)
                {
                    entry.axes.left.x = gamepad.value(GilrsAxis::LeftStickX);
                    entry.axes.left.y = gamepad.value(GilrsAxis::LeftStickY);
                    entry.axes.right.x = gamepad.value(GilrsAxis::RightStickX);
                    entry.axes.right.y = gamepad.value(GilrsAxis::RightStickY);
                    entry.axes.trigger_left = gamepad.value(GilrsAxis::LeftZ);
                    entry.axes.trigger_right = gamepad.value(GilrsAxis::RightZ);

                    entry.buttons.clear();

                    let mut collected = Vec::new();

                    for button in [
                        GilrsButton::South,
                        GilrsButton::East,
                        GilrsButton::West,
                        GilrsButton::North,
                        GilrsButton::C,
                        GilrsButton::Z,
                        GilrsButton::LeftTrigger,
                        GilrsButton::LeftTrigger2,
                        GilrsButton::RightTrigger,
                        GilrsButton::RightTrigger2,
                        GilrsButton::Select,
                        GilrsButton::Start,
                        GilrsButton::Mode,
                        GilrsButton::LeftThumb,
                        GilrsButton::RightThumb,
                        GilrsButton::DPadUp,
                        GilrsButton::DPadDown,
                        GilrsButton::DPadLeft,
                        GilrsButton::DPadRight,
                    ]
                    {
                        if gamepad.is_pressed(button)
                        {
                            collected.push(button);
                        }
                    }

                    collected.sort_by_key(|stored| *stored as u16);
                    entry.buttons = collected;
                }
            }
        }

        fn handle_event(&mut self, event: GilrsEvent)
        {
            match event.event
            {
                GilrsEventType::Connected =>
                {
                    self.add_controller(event.id);
                    self.refresh_state(event.id);
                }
                GilrsEventType::Disconnected => self.remove_controller(event.id),
                GilrsEventType::ButtonPressed(button, _) =>
                {
                    self.update_button(event.id, button, true);
                }
                GilrsEventType::ButtonReleased(button, _) =>
                {
                    self.update_button(event.id, button, false);
                }
                GilrsEventType::AxisChanged(axis, value, _) =>
                {
                    self.update_axis(event.id, axis, value);
                }
                GilrsEventType::Dropped => self.refresh_state(event.id),
                _ => {}
            }
        }

        pub fn compose_display(&self) -> String
        {
            if self.controllers.is_empty()
            {
                return "No controllers connected".to_string();
            }

            let mut entries: Vec<&ControllerEntry> = self.controllers.values().collect();
            entries.sort_by_key(|entry| entry.index);

            let mut output = String::new();

            for (idx, entry) in entries.iter().enumerate()
            {
                if idx > 0
                {
                    output.push('\n');
                }

                let _ = write!(output, "Controller{}: ", entry.index);

                if entry.buttons.is_empty()
                {
                    output.push_str("Buttons[] ");
                }
                else
                {
                    output.push_str("Buttons[");

                    for (button_idx, button) in entry.buttons.iter().enumerate()
                    {
                        if button_idx > 0
                        {
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

    impl Default for GamepadManager
    {
        fn default() -> Self
        {
            Self::new()
        }
    }

    fn button_label(button: GilrsButton) -> &'static str
    {
        match button
        {
            GilrsButton::South => "A",
            GilrsButton::East => "B",
            GilrsButton::West => "X",
            GilrsButton::North => "Y",
            GilrsButton::C => "C",
            GilrsButton::Z => "Z",
            GilrsButton::Select => "Back",
            GilrsButton::Mode => "Guide",
            GilrsButton::Start => "Start",
            GilrsButton::LeftThumb => "LStick",
            GilrsButton::RightThumb => "RStick",
            GilrsButton::LeftTrigger => "L",
            GilrsButton::RightTrigger => "R",
            GilrsButton::LeftTrigger2 => "L2",
            GilrsButton::RightTrigger2 => "R2",
            GilrsButton::DPadUp => "DPadUp",
            GilrsButton::DPadDown => "DPadDown",
            GilrsButton::DPadLeft => "DPadLeft",
            GilrsButton::DPadRight => "DPadRight",
            GilrsButton::Unknown => "Unknown",
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod gamepad
{
    use bevy::prelude::Resource;

    #[derive(Resource)]
    pub struct GamepadManager;

    impl GamepadManager
    {
        pub fn new() -> Self
        {
            Self
        }

        pub fn poll_events(&mut self)
        {
        }

        pub fn compose_display(&self) -> String
        {
            "Controller support is unavailable on this platform".to_string()
        }
    }
}

use gamepad::GamepadManager;

#[cfg(target_os = "windows")]
fn process_controller_events(
    mut manager: Local<GamepadManager>,
    mut display: Query<&mut Text, With<ControllerDisplay>>,
)
{
    manager.poll_events();

    if let Ok(mut text) = display.get_single_mut()
    {
        text.sections[0].value = manager.compose_display();
    }
}

#[cfg(not(target_os = "windows"))]
fn process_controller_events(
    mut manager: ResMut<GamepadManager>,
    mut display: Query<&mut Text, With<ControllerDisplay>>,
)
{
    manager.poll_events();

    if let Ok(mut text) = display.get_single_mut()
    {
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
