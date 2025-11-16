use bevy::prelude::*;
use crate::core::units::Seconds;

pub struct TimeManagerPlugin;

pub const TICK_DURATION: Seconds = Seconds(0.01);

#[allow(dead_code)]
#[derive(Event)]
pub struct TickEvent
{
    pub tick: u64,
}

#[allow(dead_code)]
#[derive(Event)]
pub struct TenTickEvent
{
    pub tick: u64,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum TimeSystem
{
    Accumulate,
    Notify,
}

impl Plugin for TimeManagerPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_event::<TickEvent>()
            .add_event::<TenTickEvent>()
            .init_resource::<SimulationTime>()
            .configure_sets(Update, (TimeSystem::Accumulate, TimeSystem::Notify).chain())
            .add_systems(Update, accumulate_time.in_set(TimeSystem::Accumulate))
            .add_systems(Update, dispatch_ten_tick_events.in_set(TimeSystem::Notify));
    }
}

#[derive(Resource)]
pub struct SimulationTime
{
    pub elapsed: Seconds,
    pub ticks: u64,
    sub_tick: Seconds,
}

impl Default for SimulationTime
{
    fn default() -> Self
    {
        Self
        {
            elapsed: Seconds::default(),
            ticks: 0,
            sub_tick: Seconds::default(),
        }
    }
}

fn accumulate_time(
    time: Res<Time>,
    mut simulation_time: ResMut<SimulationTime>,
    mut tick_events: EventWriter<TickEvent>
)
{
    let delta_seconds = Seconds::from(time.delta_seconds());

    simulation_time.elapsed += delta_seconds;
    simulation_time.sub_tick += delta_seconds;

    let tick_length = TICK_DURATION.value();

    if simulation_time.sub_tick.value() < tick_length
    {
        return;
    }

    let completed_ticks = (simulation_time.sub_tick.value() / tick_length).floor() as u64;

    if completed_ticks == 0
    {
        return;
    }

    let start_tick = simulation_time.ticks + 1;

    simulation_time.ticks += completed_ticks;
    simulation_time.sub_tick -= Seconds::from(tick_length * completed_ticks as f32);

    for tick in start_tick..=simulation_time.ticks
    {
        tick_events.send(TickEvent { tick });
    }
}

fn dispatch_ten_tick_events(
    simulation_time: Res<SimulationTime>,
    mut last_group: Local<u64>,
    mut event_writer: EventWriter<TenTickEvent>,
)
{
    let current_group = simulation_time.ticks / 10;

    if current_group <= *last_group
    {
        return;
    }

    for index in (*last_group + 1)..=current_group
    {
        let tick_value = index * 10;
        event_writer.send(TenTickEvent { tick: tick_value });
    }

    *last_group = current_group;
}
