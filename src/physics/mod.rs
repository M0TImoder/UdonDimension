pub mod world;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_systems(Startup, world::spawn_world);
    }
}
