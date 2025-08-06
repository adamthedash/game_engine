use std::{fmt::Debug, time::Duration};

use crate::{
    entity::{
        component_manager::ComponentManager, components::EntityType, entity_manager::EntityManager,
        systems::move_system,
    },
    state::world::WorldPos,
};

pub mod component_manager;
pub mod components;
pub mod entity_manager;
pub mod systems;

pub type EntityId = usize;

#[derive(Debug)]
pub struct SpawnEntityMessage {
    pub pos: WorldPos,
    pub entity_type: EntityType,
}

pub struct ECS {
    pub entity_manager: EntityManager,
    pub component_manager: ComponentManager,
}

impl ECS {
    pub fn new(max_entities: usize) -> Self {
        Self {
            entity_manager: EntityManager::new(max_entities),
            component_manager: ComponentManager::default(),
        }
    }

    pub fn tick(&mut self, duration: &Duration) {
        let (positions, orientations) = self.component_manager.get_component_mut2();
        move_system(positions, orientations, duration);
    }
}
