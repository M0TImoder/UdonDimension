use bevy::prelude::*;

pub struct TimeManagerPlugin;

impl Plugin for TimeManagerPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<SimulationTime>()
            .add_systems(Update, accumulate_time);
    }
}

#[derive(Resource, Default)]
pub struct SimulationTime
{
    pub elapsed: f32,
}

fn accumulate_time(time: Res<Time>, mut simulation_time: ResMut<SimulationTime>)
{
    simulation_time.elapsed += time.delta_seconds();
}
