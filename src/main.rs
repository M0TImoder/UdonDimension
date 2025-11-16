mod core;
mod physics;
mod robot;
mod debug;

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
                core::time::TimeManagerPlugin,
                core::CorePlugin,
                robot::RobotPlugin,
                debug::DebugPlugin
            )
        )
        .run();
}
