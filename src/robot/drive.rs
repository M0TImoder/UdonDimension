use bevy::prelude::*;
use crate::core::units::{MetersPerSecond, Rpm};

#[allow(dead_code)]
#[derive(Component, Default)]
pub struct DriveInput
{
    pub vx: MetersPerSecond,
    pub vy: MetersPerSecond,
    pub omega: Rpm,
}
