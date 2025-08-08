use std::fmt::Debug;

use crate::{entity::components::EntityType, state::world::WorldPos};

pub mod components;
pub mod systems;
pub mod bundles;

pub type EntityId = usize;

#[derive(Debug)]
pub struct SpawnEntityMessage {
    pub pos: WorldPos,
    pub entity_type: EntityType,
}
