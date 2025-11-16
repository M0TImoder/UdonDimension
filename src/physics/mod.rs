pub mod drag;
pub mod world;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.insert_resource(
                RapierConfiguration
                {
                    gravity: Vec3::new(0.0, -drag::STANDARD_GRAVITY, 0.0),
                    ..default()
                }
            )
            .insert_resource(drag::AirEnvironment::default())
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_systems(Startup, world::spawn_world)
            .add_systems(Update, (drag::update_air_environment, drag::apply_aerodynamic_drag));
    }
}
