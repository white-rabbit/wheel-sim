use crate::components::*;
use bevy::prelude::{Bundle, Vec2};

#[derive(Bundle)]
pub struct WheelBundle {
    pub radius: Radius,
    pub mass: Mass,
    pub pos: Pos,
    pub vel: Vel,
    pub rot: Rot,
    pub rot_vel: RotVel,
    pub state: ContactState,
}

impl WheelBundle {
    pub fn new(pos: Vec2, vel: Vec2) -> Self {
        Self {
            radius: Radius(50.0),
            mass: Mass(1.0),
            pos: Pos(pos),
            vel: Vel(vel),
            rot: Rot(0.0),
            rot_vel: RotVel(0.0),
            state: ContactState {
                has_contact: false,
                contact_pos: Vec2::ZERO,
                sliding_vel: Vec2::ZERO,
            },
        }
    }
}
