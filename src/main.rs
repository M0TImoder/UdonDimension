mod core;
mod physics;
mod robot;
mod debug;
mod design;
mod ui;

use bevy::prelude::*;
use bevy::window::WindowPlugin;

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
                physics::PhysicsPlugin,
                design::DesignPlugin,
                core::time::TimeManagerPlugin,
                core::CorePlugin,
                robot::RobotPlugin,
                debug::DebugPlugin,
                ui::UiPlugin
                // bevy_rapier3d::render::RapierDebugRenderPlugin::default()
            )
        )
        .run();
}
