use std::ops::{AddAssign, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct Rpm(pub f32);

#[allow(dead_code)]
impl Rpm
{
    pub fn new(value_rpm: f32) -> Self
    {
        Self(value_rpm)
    }

    pub fn value(self) -> f32
    {
        self.0
    }
}

impl From<f32> for Rpm
{
    fn from(value: f32) -> Self
    {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct Meters(pub f32);

#[allow(dead_code)]
impl Meters
{
    pub fn new(value_m: f32) -> Self
    {
        Self(value_m)
    }

    pub fn value(self) -> f32
    {
        self.0
    }
}

impl From<f32> for Meters
{
    fn from(value: f32) -> Self
    {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct MetersPerSecond(pub f32);

#[allow(dead_code)]
impl MetersPerSecond
{
    pub fn new(value_mps: f32) -> Self
    {
        Self(value_mps)
    }

    pub fn value(self) -> f32
    {
        self.0
    }
}

impl From<f32> for MetersPerSecond
{
    fn from(value: f32) -> Self
    {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct Kilograms(pub f32);

#[allow(dead_code)]
impl Kilograms
{
    pub fn new(value_kg: f32) -> Self
    {
        Self(value_kg)
    }

    pub fn value(self) -> f32
    {
        self.0
    }
}

impl From<f32> for Kilograms
{
    fn from(value: f32) -> Self
    {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct Seconds(pub f32);

#[allow(dead_code)]
impl Seconds
{
    pub fn new(value_s: f32) -> Self
    {
        Self(value_s)
    }

    pub fn value(self) -> f32
    {
        self.0
    }
}

impl From<f32> for Seconds
{
    fn from(value: f32) -> Self
    {
        Self::new(value)
    }
}

impl AddAssign for Seconds
{
    fn add_assign(&mut self, rhs: Self)
    {
        self.0 += rhs.0;
    }
}

impl SubAssign for Seconds
{
    fn sub_assign(&mut self, rhs: Self)
    {
        self.0 -= rhs.0;
    }
}
