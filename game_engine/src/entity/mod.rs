use std::{collections::VecDeque, time::Duration};

use crate::{
    entity::{
        components::{EntityType, Orientation, Position},
        systems::move_system,
    },
    state::world::WorldPos,
};

pub mod components;
pub mod systems;

pub type EntityId = usize;

pub struct EntityManager {
    free_ids: VecDeque<EntityId>,
}

impl EntityManager {
    pub fn new(max_entities: usize) -> Self {
        Self {
            free_ids: (0..max_entities).collect(),
        }
    }

    pub fn create_entity(&mut self) -> EntityId {
        self.free_ids.pop_front().expect("Ran out of entity slots!")
    }

    pub fn destroy_entity(&mut self, id: EntityId) {
        // TODO: Check that an entity hasn't been double-destroyed?
        self.free_ids.push_back(id);
    }
}

pub struct ComponentManager {
    // Mapping between EntityId -> Component index
    entity_indices: Vec<Option<usize>>,
    // Components
    pub positions: Vec<Position>,
    pub orientations: Vec<Orientation>,
}

impl ComponentManager {
    pub fn new(max_entities: usize) -> Self {
        Self {
            entity_indices: vec![None; max_entities],
            positions: Default::default(),
            orientations: Default::default(),
        }
    }

    pub fn create_entity(&mut self, id: EntityId) {
        assert!(
            self.entity_indices[id].is_none(),
            "Entity already initalised!"
        );

        // New entities go on the end
        let index = self.num_entities();
        self.entity_indices[id] = Some(index);

        self.positions.push(Default::default());
        self.orientations.push(Default::default());
    }

    pub fn delete_entity(&mut self, id: EntityId) {
        let index = self.get_entity_index(id);

        self.positions.swap_remove(index);
        self.orientations.swap_remove(index);

        let num_entities = self.num_entities();

        // Identify which index was swapped and update accordingly
        // TODO: maintain separate vec to map back instead of searching each time
        let swapped_index = self
            .entity_indices
            .iter_mut()
            .find(|i| i.is_some_and(|i| i == num_entities))
            .expect("Swapped entity not found!");

        *swapped_index = Some(index);
    }

    /// The number of entities currently initalised
    pub fn num_entities(&self) -> usize {
        self.positions.len()
    }

    pub fn get_entity_index(&self, id: EntityId) -> usize {
        self.entity_indices[id].expect("Entity hasn't been initalised!")
    }
}

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
            component_manager: ComponentManager::new(max_entities),
        }
    }

    pub fn tick(&mut self, duration: &Duration) {
        move_system(
            &mut self.component_manager.positions,
            &mut self.component_manager.orientations,
            duration,
        );
    }
}
