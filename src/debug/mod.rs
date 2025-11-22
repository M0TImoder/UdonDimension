use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy_rapier3d::render::DebugRenderContext;
use bevy::input::gamepad::{
    Gamepad,
    GamepadAxis,
    GamepadAxisType,
    GamepadButton,
    GamepadButtonType,
    Gamepads,
};
use bevy::input::Axis;
use bevy::prelude::*;

pub struct DebugPlugin;

#[derive(Resource)]
struct DebugOverlayState
{
    visible: bool,
}

impl Default for DebugOverlayState
{
    fn default() -> Self
    {
        Self
        {
            visible: true,
        }
    }
}

#[derive(Resource)]
struct RapierDebugState
{
    enabled: bool,
}

impl Default for RapierDebugState
{
    fn default() -> Self
    {
        Self
        {
            enabled: false,
        }
    }
}

#[derive(Component)]
struct DebugHudElement;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct PositionText;

#[derive(Component)]
struct GamepadText;

const MONITORED_BUTTONS: [GamepadButtonType; 14] =
[
    GamepadButtonType::DPadUp,
    GamepadButtonType::DPadRight,
    GamepadButtonType::DPadLeft,
    GamepadButtonType::DPadDown,
    GamepadButtonType::West,
    GamepadButtonType::South,
    GamepadButtonType::North,
    GamepadButtonType::East,
    GamepadButtonType::LeftTrigger,
    GamepadButtonType::RightTrigger,
    GamepadButtonType::LeftTrigger2,
    GamepadButtonType::RightTrigger2,
    GamepadButtonType::LeftThumb,
    GamepadButtonType::RightThumb
];

const MONITORED_AXES: [GamepadAxisType; 4] =
[
    GamepadAxisType::LeftStickX,
    GamepadAxisType::LeftStickY,
    GamepadAxisType::RightStickX,
    GamepadAxisType::RightStickY
];

const GAMEPAD_LINE_INDENT: &str = "                  ";

impl Plugin for DebugPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .init_resource::<DebugOverlayState>()
            .init_resource::<RapierDebugState>()
            .add_systems(Startup, (setup_debug_overlay, setup_rapier_debug))
            .add_systems(
                Update,
                (
                    toggle_debug_overlay,
                    toggle_rapier_debug,
                    apply_debug_visibility.after(toggle_debug_overlay),
                    apply_rapier_debug_visibility.after(toggle_rapier_debug),
                    update_fps_text,
                    update_position_text,
                    update_gamepad_text
                )
            );
    }
}

fn setup_debug_overlay(mut commands: Commands, asset_server: Res<AssetServer>)
{
    let font = asset_server.load("fonts/NotoSansJP-Medium.ttf");

    commands.spawn(
        (
            TextBundle
            {
                style: Style
                {
                    position_type: PositionType::Absolute,
                    top: Val::Px(8.0),
                    left: Val::Px(8.0),
                    ..default()
                },
                text: Text::from_sections(
                    [
                        TextSection::new(
                            "FPS: ",
                            TextStyle
                            {
                                font: font.clone(),
                                font_size: 16.0,
                                color: Color::WHITE,
                            }
                        ),
                        TextSection::new(
                            "0",
                            TextStyle
                            {
                                font: font.clone(),
                                font_size: 16.0,
                                color: Color::WHITE,
                            }
                        )
                    ]
                ),
                ..default()
            },
            DebugHudElement,
            FpsText
        )
    );

    commands.spawn(
        (
            TextBundle
            {
                style: Style
                {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(8.0),
                    left: Val::Px(8.0),
                    ..default()
                },
                text: Text::from_sections(
                    [
                        TextSection::new(
                            "座標: ",
                            TextStyle
                            {
                                font: font.clone(),
                                font_size: 16.0,
                                color: Color::WHITE,
                            }
                        ),
                        TextSection::new(
                            "0.00, 0.00, 0.00",
                            TextStyle
                            {
                                font: font.clone(),
                                font_size: 16.0,
                                color: Color::WHITE,
                            }
                        )
                    ]
                ),
                ..default()
            },
            DebugHudElement,
            PositionText
        )
    );

    commands.spawn(
        (
            TextBundle
            {
                style: Style
                {
                    position_type: PositionType::Absolute,
                    top: Val::Px(8.0),
                    right: Val::Px(8.0),
                    ..default()
                },
                text: Text::from_sections(
                    [
                        TextSection::new(
                            "コントローラ:\n",
                            TextStyle
                            {
                                font: font.clone(),
                                font_size: 16.0,
                                color: Color::WHITE,
                            }
                        ),
                        TextSection::new(
                            "未接続",
                            TextStyle
                            {
                                font: font,
                                font_size: 16.0,
                                color: Color::WHITE,
                            }
                        )
                    ]
                )
                .with_justify(JustifyText::Right),
                ..default()
            },
            DebugHudElement,
            GamepadText
        )
    );
}

fn setup_rapier_debug(mut debug_context: ResMut<DebugRenderContext>)
{
    debug_context.enabled = false;
}

fn toggle_debug_overlay(input: Res<ButtonInput<KeyCode>>, mut state: ResMut<DebugOverlayState>)
{
    if input.just_pressed(KeyCode::F2)
    {
        state.visible = !state.visible;
    }
}

fn toggle_rapier_debug(input: Res<ButtonInput<KeyCode>>, mut state: ResMut<RapierDebugState>)
{
    if input.just_pressed(KeyCode::F1)
    {
        state.enabled = !state.enabled;
    }
}

fn apply_rapier_debug_visibility(
    state: Res<RapierDebugState>,
    mut debug_context: ResMut<DebugRenderContext>
)
{
    debug_context.enabled = state.enabled;
}

fn apply_debug_visibility(
    state: Res<DebugOverlayState>,
    mut query: Query<&mut Visibility, With<DebugHudElement>>
)
{
    let visibility = if state.visible
    {
        Visibility::Visible
    }
    else
    {
        Visibility::Hidden
    };

    for mut element_visibility in query.iter_mut()
    {
        *element_visibility = visibility;
    }
}

fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>
)
{
    let mut text = match query.get_single_mut()
    {
        Ok(text) => text,
        Err(_) => return,
    };

    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps_diag| fps_diag.smoothed())
    {
        text.sections[1].value = format!("{:.0}", fps);
    }
}

fn update_position_text(
    mut query: Query<&mut Text, With<PositionText>>,
    camera_query: Query<&Transform, With<Camera>>
)
{
    let mut text = match query.get_single_mut()
    {
        Ok(text) => text,
        Err(_) => return,
    };

    let transform = match camera_query.get_single()
    {
        Ok(transform) => transform,
        Err(_) => return,
    };

    let position = transform.translation;
    text.sections[1].value = format!("{:.2}, {:.2}, {:.2}", position.x, position.y, position.z);
}

fn update_gamepad_text(
    gamepads: Res<Gamepads>,
    buttons: Res<ButtonInput<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut query: Query<&mut Text, With<GamepadText>>
)
{
    let mut text = match query.get_single_mut()
    {
        Ok(text) => text,
        Err(_) => return,
    };

    let base_style = match text.sections.first()
    {
        Some(section) => section.style.clone(),
        None => return,
    };

    let mut sections = Vec::new();
    sections.push(TextSection::new("コントローラ:\n", base_style.clone()));

    if gamepads.iter().next().is_none()
    {
        sections.push(TextSection::new("未接続", base_style));
        text.sections = sections;
        return;
    }

    let collected_gamepads: Vec<Gamepad> = gamepads.iter().collect();

    for (index, gamepad) in collected_gamepads.iter().enumerate()
    {
        let status_line = format!("コントローラ[{}]: 接続中\n", gamepad.id);
        sections.push(TextSection::new(status_line, base_style.clone()));

        sections.extend(button_line_sections(*gamepad, &buttons, &base_style));

        let stick = collect_stick_axes(*gamepad, &axes);
        sections.push(TextSection::new(
            format!(
                "{}LStick: X = {:>7.1} | Y = {:>7.1}\n",
                GAMEPAD_LINE_INDENT,
                stick.left_x,
                stick.left_y
            ),
            base_style.clone()
        ));

        let line_end = if index + 1 == collected_gamepads.len()
        {
            ""
        }
        else
        {
            "\n"
        };

        sections.push(TextSection::new(
            format!(
                "{}RStick: X = {:>7.1} | Y = {:>7.1}{}",
                GAMEPAD_LINE_INDENT,
                stick.right_x,
                stick.right_y,
                line_end
            ),
            base_style.clone()
        ));
    }

    text.sections = sections;
}

fn button_line_sections(
    gamepad: Gamepad,
    buttons: &ButtonInput<GamepadButton>,
    base_style: &TextStyle
) -> Vec<TextSection>
{
    let mut sections = Vec::new();
    sections.push(TextSection::new(GAMEPAD_LINE_INDENT, base_style.clone()));

    for button_type in MONITORED_BUTTONS
    {
        let pressed = buttons.pressed(GamepadButton
        {
            gamepad,
            button_type,
        });

        sections.push(button_section(button_type, pressed, base_style));
    }

    sections.push(TextSection::new("\n", base_style.clone()));

    sections
}

fn button_section(
    button_type: GamepadButtonType,
    pressed: bool,
    base_style: &TextStyle
) -> TextSection
{
    let mut style = base_style.clone();

    style.color = if pressed
    {
        Color::RED
    }
    else
    {
        Color::WHITE
    };

    let mut label = button_label(button_type).to_string();

    if button_type != GamepadButtonType::RightThumb
    {
        label.push(' ');
    }

    TextSection::new(label, style)
}

struct StickAxes
{
    left_x: f32,
    left_y: f32,
    right_x: f32,
    right_y: f32,
}

fn collect_stick_axes(gamepad: Gamepad, axes: &Axis<GamepadAxis>) -> StickAxes
{
    let mut left_x = 0.0;
    let mut left_y = 0.0;
    let mut right_x = 0.0;
    let mut right_y = 0.0;

    for axis_type in MONITORED_AXES
    {
        let scaled = scaled_axis_value(gamepad, axes, axis_type);

        match axis_type
        {
            GamepadAxisType::LeftStickX => left_x = scaled,
            GamepadAxisType::LeftStickY => left_y = scaled,
            GamepadAxisType::RightStickX => right_x = scaled,
            GamepadAxisType::RightStickY => right_y = scaled,
            _ => (),
        }
    }

    StickAxes
    {
        left_x,
        left_y,
        right_x,
        right_y,
    }
}

fn scaled_axis_value(
    gamepad: Gamepad,
    axes: &Axis<GamepadAxis>,
    axis_type: GamepadAxisType
) -> f32
{
    let axis = GamepadAxis
    {
        gamepad,
        axis_type,
    };

    let raw = axes.get(axis).unwrap_or(0.0);

    (raw * 255.0).clamp(-255.0, 255.0)
}

fn button_label(button_type: GamepadButtonType) -> &'static str
{
    match button_type
    {
        GamepadButtonType::South => "A",
        GamepadButtonType::East => "B",
        GamepadButtonType::West => "X",
        GamepadButtonType::North => "Y",
        GamepadButtonType::C => "C",
        GamepadButtonType::Z => "Z",
        GamepadButtonType::LeftTrigger => "L1",
        GamepadButtonType::LeftTrigger2 => "L2",
        GamepadButtonType::RightTrigger => "R1",
        GamepadButtonType::RightTrigger2 => "R2",
        GamepadButtonType::Select => "Select",
        GamepadButtonType::Start => "Start",
        GamepadButtonType::Mode => "Mode",
        GamepadButtonType::LeftThumb => "L3",
        GamepadButtonType::RightThumb => "R3",
        GamepadButtonType::DPadUp => "上",
        GamepadButtonType::DPadDown => "下",
        GamepadButtonType::DPadLeft => "左",
        GamepadButtonType::DPadRight => "右",
        _ => "不明",
    }
}
