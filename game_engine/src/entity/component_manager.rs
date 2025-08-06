use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

use rustc_hash::FxHashMap;

use crate::entity::EntityId;

/// Trait used so that we can store generic Vec<T>'s but downcast them to concrete types when
/// accessed
pub trait Component: Any + Debug + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn expand_capacity(&mut self, capacity: usize);
    fn swap_remove(&mut self, index: usize);
}

impl<T: Default + Debug + 'static> Component for Vec<T> {
    fn expand_capacity(&mut self, capacity: usize) {
        self.resize_with(capacity, T::default);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn swap_remove(&mut self, index: usize) {
        self.swap_remove(index);
    }
}

#[derive(Debug, Default)]
pub struct ComponentManager {
    // Mapping between EntityId -> Component index
    entity_indices: Vec<Option<usize>>,
    // Mapping between Component index -> EntityId
    index_entity: Vec<EntityId>,
    // Holds Vec<T>'s
    components: FxHashMap<TypeId, Box<dyn Component>>,
}

impl ComponentManager {
    /// Add a new entity with default values for components
    pub fn add_entity(&mut self, id: EntityId) {
        if id >= self.entity_indices.len() {
            // Extend the capacity of the manager to the given amount
            self.entity_indices.resize_with(id + 1, Default::default);
        }
        assert!(
            self.entity_indices[id].is_none(),
            "Entity already initalised!"
        );

        // Components for new entities get initialised on the end
        let index = self.num_entities();
        self.components.values_mut().for_each(|component| {
            component.expand_capacity(index + 1);
        });

        self.entity_indices[id] = Some(index);
        self.index_entity.push(id);
    }

    pub fn delete_entity(&mut self, id: EntityId) {
        // Remove the given entity's data
        let index = self.get_entity_index(id);

        self.components.values_mut().for_each(|component| {
            component.swap_remove(index);
        });
        self.entity_indices[id] = None;
        self.index_entity.swap_remove(index);

        // Identify which index was swapped and update accordingly
        if index == self.num_entities() {
            // Entity was last, nothing to update
            return;
        }

        let swapped_entity = self.index_entity[index];
        self.entity_indices[swapped_entity] = Some(index);
    }

    /// Sets the value of a given component for a single entity
    pub fn set_entity_component_value<T: Default + Debug + Clone + 'static>(
        &mut self,
        id: EntityId,
        value: T,
    ) {
        // Add new component if needed
        let type_id = TypeId::of::<T>();
        if !self.components.contains_key(&type_id) {
            self.add_component_type::<T>();
        }

        let index = self.entity_indices[id].expect("Entity not initalised!");
        self.get_component_mut()[index] = value;
    }

    /// The number of entities currently initalised
    pub fn num_entities(&self) -> usize {
        self.index_entity.len()
    }

    pub fn get_entity_index(&self, id: EntityId) -> usize {
        self.entity_indices
            .get(id)
            .expect("Entity hasn't been initalised!")
            .expect("Entity hasn't been initalised!")
    }

    /// Get a component vector
    pub fn get_component<T: 'static>(&self) -> &Vec<T> {
        let id = TypeId::of::<T>();
        self.components
            .get(&id)
            .expect("Component of the given type not registered!")
            .as_any()
            .downcast_ref()
            .expect("Failed to cast component, this should never happen!")
    }

    fn get_components_mut_many<const N: usize>(&mut self, ids: [&TypeId; N]) -> [&mut dyn Any; N] {
        self.components.get_disjoint_mut(ids).map(|component| {
            component
                .expect("Component of the given type not registered!")
                .as_any_mut()
        })
    }

    /// Get a mutable component vector
    pub fn get_component_mut<T: 'static>(&mut self) -> &mut Vec<T> {
        let id = TypeId::of::<T>();
        self.components
            .get_mut(&id)
            .expect("Component of the given type not registered!")
            .as_any_mut()
            .downcast_mut()
            .expect("Failed to cast component, this should never happen!")
    }

    /// TODO: Figure out how to generically do this
    pub fn get_component_mut2<T1: 'static, T2: 'static>(&mut self) -> (&mut Vec<T1>, &mut Vec<T2>) {
        let id1 = TypeId::of::<T1>();
        let id2 = TypeId::of::<T2>();
        let [any1, any2] = self.get_components_mut_many([&id1, &id2]);

        let concrete1 = any1
            .downcast_mut()
            .expect("Failed to cast component, this should never happen!");
        let concrete2 = any2
            .downcast_mut()
            .expect("Failed to cast component, this should never happen!");

        (concrete1, concrete2)
    }

    /// Register a new type of component for the system
    pub fn add_component_type<T: Default + Debug + Clone + 'static>(&mut self) {
        let id = TypeId::of::<T>();
        self.components
            .insert(id, Box::new(vec![T::default(); self.num_entities()]));
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::ComponentManager;

    #[test]
    fn test_component() {
        let mut manager = ComponentManager::default();

        manager.add_component_type::<&'static str>();
        manager.add_component_type::<i32>();
        println!("{manager:?}");

        let name = "Dave";
        let age = 13;
        let entity_id = 2;
        manager.add_entity(entity_id);
        manager.set_entity_component_value(entity_id, age);
        manager.set_entity_component_value(entity_id, name);

        let (names, ages) = manager.get_component_mut2::<&'static str, i32>();
        names.iter().zip(ages).for_each(|(name, age)| {
            println!("{name:?} {age:?}");
        });

        println!("{manager:?}");
    }
}
