use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Pos(pub Vec2);

#[derive(Component, Debug, Default)]
pub struct Vel(pub Vec2);

#[derive(Component, Debug)]
pub struct Radius(pub f32);

#[derive(Component, Debug)]
pub struct Mass(pub f32);

#[derive(Component, Debug, Default)]
pub struct Rot(pub f32);

#[derive(Component, Debug, Default)]
pub struct RotVel(pub f32);

#[derive(Component, Debug)]
pub struct ContactState {
    pub has_contact: bool,
    pub contact_pos: Vec2,
    pub sliding_vel: Vec2,
}

impl Default for Radius {
    fn default() -> Self {
        Self(100.) // Default to 100 cm
    }
}

#[derive(Component, Debug)]
pub struct Particle(pub f32);

impl Default for Mass {
    fn default() -> Self {
        Self(1.) // Default to 1 kg
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self(60.0) // Default to 60 frames.
    }
}
