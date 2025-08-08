use cgmath::{Point3, Quaternion, Rad, Zero};
use enum_map::EnumMap;

use crate::{
    data::{item::ItemType, recipe::Recipe},
    entity::EntityId,
    state::world::WorldPos,
};

#[derive(Debug)]
pub enum EntityType {
    Sibeal,
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

pub struct Container {
    pub items: EnumMap<ItemType, usize>,
}

impl Container {
    pub fn add_item(&mut self, item: ItemType, count: usize) {
        self.items[item] += count;
    }

    pub fn remove_item(&mut self, item: ItemType, count: usize) {
        assert!(self.items[item] >= count, "Not enough items!");

        self.items[item] -= count;
    }
}

pub struct Crafter {
    pub recipe: Option<Recipe>,
    pub crafting_juice: f32,
    pub juice_per_second: f32,
}

pub struct UprightOrientation {
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}
