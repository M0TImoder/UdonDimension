use bevy::prelude::*;

#[allow(dead_code)]
#[derive(Component, Default)]
pub struct DriveInput
{
    pub vx: f32,
    pub vy: f32,
    pub omega: f32,
}
