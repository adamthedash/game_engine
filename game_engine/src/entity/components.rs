use cgmath::{Point3, Quaternion, Zero};

use crate::{entity::EntityId, state::world::WorldPos};

#[derive(Debug)]
pub enum EntityType {
    Sibeal,
}

pub struct Health {
    health: f32,
}

#[derive(Debug, Clone)]
pub struct Position(pub WorldPos);

impl Default for Position {
    fn default() -> Self {
        Self(WorldPos(Point3::new(0., 0., 0.)))
    }
}

#[derive(Debug, Clone)]
pub struct Orientation(pub Quaternion<f32>);

impl Default for Orientation {
    fn default() -> Self {
        Self(Quaternion::zero())
    }
}

pub struct Vision {
    distance: f32,
}

pub struct Movement {
    speed: f32,
}

pub enum Behaviour {
    Idle,
    Wandering(WorldPos),
    Persuing(EntityId),
}
