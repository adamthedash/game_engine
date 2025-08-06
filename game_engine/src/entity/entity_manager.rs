use std::collections::VecDeque;

use crate::entity::EntityId;

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
