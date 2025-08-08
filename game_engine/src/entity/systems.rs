use std::time::Duration;

use cgmath::{Rotation, Vector3};
use hecs::World;

use crate::entity::components::{Container, Crafter, Orientation, Position};

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
    for (id, (container, crafter)) in ecs.query_mut::<(&mut Container, &mut Crafter)>() {
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
