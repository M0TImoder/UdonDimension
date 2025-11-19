pub mod loader;

use bevy::prelude::*;
use loader::RobotLoaderPlugin;

pub struct DesignPlugin;

impl Plugin for DesignPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RobotLoaderPlugin);
    }
}
