use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
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

#[derive(Component)]
struct DebugHudElement;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct PositionText;

impl Plugin for DebugPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .init_resource::<DebugOverlayState>()
            .add_systems(Startup, setup_debug_overlay)
            .add_systems(
                Update,
                (
                    toggle_debug_overlay,
                    apply_debug_visibility.after(toggle_debug_overlay),
                    update_fps_text,
                    update_position_text
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
                                font: font,
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
}

fn toggle_debug_overlay(input: Res<ButtonInput<KeyCode>>, mut state: ResMut<DebugOverlayState>)
{
    if input.just_pressed(KeyCode::F2)
    {
        state.visible = !state.visible;
    }
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
