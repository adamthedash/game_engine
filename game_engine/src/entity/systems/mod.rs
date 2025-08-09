pub mod ui;

use std::time::Duration;

use cgmath::{Rotation, Vector3};
use hecs::{Entity, EntityBuilder, World};

use crate::{
    data::{
        block::BlockType,
        item::ItemType,
        loader::{BLOCKS, ITEMS},
    },
    entity::components::{Container, Crafter, Hotbar, Orientation, Position},
    state::{game::TransferItemMessage, world::BlockPos},
};

pub trait System {
    fn tick(eccs: &mut World, duration: &Duration);
}

pub struct MoveSystem;

impl System for MoveSystem {
    fn tick(ecs: &mut World, duration: &Duration) {
        for (_, (position, orientation)) in ecs.query_mut::<(&mut Position, &mut Orientation)>() {
            let facing = orientation.0.rotate_vector(Vector3::unit_z());
            let movement_speed = 0.1 * duration.as_secs_f32();

            position.0.0 += facing * movement_speed;
        }
    }
}

/// Tick all crafters
pub fn crafting_tick(ecs: &mut World, duration: &Duration) {
    for (_, (container, crafter)) in ecs.query_mut::<(&mut Container, &mut Crafter)>() {
        // Only process when we've got a recipe
        let Some(recipe) = &crafter.recipe else {
            return;
        };

        // Only process when we've got enough materials
        let have_materials = recipe
            .inputs
            .iter()
            .all(|(item, amount)| container.items[*item] >= *amount);
        if !have_materials {
            return;
        }

        // Make some progress on the recipe
        crafter.crafting_juice += crafter.juice_per_second * duration.as_secs_f32();

        // Craft an item if we've got enough
        if crafter.crafting_juice >= recipe.crafting_juice_cost {
            crafter.crafting_juice -= recipe.crafting_juice_cost;

            recipe.inputs.iter().for_each(|(&item, &amount)| {
                container.remove_item(item, amount);
            });

            container.add_item(recipe.output.0, recipe.output.1);
        }
    }
}

/// Creates a block with the default state
pub fn create_block_state(ecs: &mut World, pos: &BlockPos, block_type: BlockType) -> Entity {
    let blocks = BLOCKS.get().expect("Block data not initialised!");

    let Some(state_fn) = blocks[block_type].data.state else {
        panic!("Attempt to spawn block state for un-stateful block!");
    };

    let mut entity = EntityBuilder::new();
    entity.add(pos.clone());
    state_fn(&mut entity);

    ecs.spawn(entity.build())
}

/// Transfer items between two sources
pub fn transfer_item(ecs: &mut World, message: &TransferItemMessage) {
    let TransferItemMessage {
        source,
        dest,
        item,
        count,
    } = *message;

    ecs.get::<&mut Container>(source)
        .expect("Failed to get source entity for item transfer")
        .remove_item(item, count);

    ecs.get::<&mut Container>(dest)
        .expect("Failed to get dest entity for item transfer")
        .add_item(item, count);
}

/// Get the block breaking strength for an entity
pub fn get_breaking_strength(ecs: &World, entity: Entity) -> u32 {
    if let Some((item, count)) = get_held_item(ecs, entity)
        && count > 0
    {
        ITEMS.get().unwrap()[item]
            .data
            .breaking_strength
            .unwrap_or(0)
    } else {
        0
    }
}

/// Get the item & count an entity is holding
pub fn get_held_item(ecs: &World, entity: Entity) -> Option<(ItemType, usize)> {
    let mut query = ecs.query_one::<(&Container, &Hotbar)>(entity).unwrap();
    let (inventory, hotbar) = query.get().unwrap();

    hotbar.slots[hotbar.selected].map(|item| (item, inventory.items[item]))
}
