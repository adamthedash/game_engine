use cgmath::{Point3, Quaternion, Rad, Vector3, Zero};
use enum_map::EnumMap;

use crate::{
    data::{
        item::ItemType,
        recipe::{RECIPES, Recipe},
    },
    entity::EntityId,
    math::angles_to_vec3,
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

/// Vision distance
pub struct Vision(pub f32);

pub struct Movement {
    speed: f32,
}

/// Reach distance
pub struct Reach(pub f32);

pub enum Behaviour {
    Idle,
    Wandering(WorldPos),
    Persuing(EntityId),
}

#[derive(Default)]
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

    /// Get the recipes the player can currently craft based on what they have on them
    pub fn get_craftable_recipes(&mut self) -> impl Iterator<Item = &'static Recipe> {
        RECIPES.iter().filter(|r| {
            r.inputs
                .iter()
                .all(|(item, count)| self.items[*item] >= *count)
        })
    }

    /// Craft the given recipe. panics if the player doesn't have eough ingredients
    pub fn craft_recipe(&mut self, recipe: &Recipe) {
        // Remove input items
        recipe.inputs.iter().for_each(|(item, count)| {
            self.remove_item(*item, *count);
        });

        // Add output items
        self.add_item(recipe.output.0, recipe.output.1);
    }
}

#[derive(Default)]
pub struct Crafter {
    pub recipe: Option<Recipe>,
    pub crafting_juice: f32,
    pub juice_per_second: f32,
}

#[derive(Clone, Debug)]
pub struct UprightOrientation {
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}

impl Default for UprightOrientation {
    fn default() -> Self {
        Self {
            yaw: Rad(0.),
            pitch: Rad(0.),
        }
    }
}

impl UprightOrientation {
    #[inline]
    pub fn forward(&self) -> Vector3<f32> {
        angles_to_vec3(self.yaw, self.pitch)
    }
}

pub enum UIType {
    Chest,
    Crafter,
}

#[derive(Default)]
pub struct Hotbar {
    // Each slot holds one item ID
    pub slots: [Option<ItemType>; 10],
    // Slot selected
    pub selected: usize,
}

impl Hotbar {
    /// Move the selected hotbar up or down by one
    pub fn scroll(&mut self, up: bool) {
        if up {
            self.selected += 1;
            self.selected %= self.slots.len();
        } else {
            if self.selected == 0 {
                self.selected += self.slots.len();
            }
            self.selected -= 1;
        }
    }

    /// Set a favourite slot to an item
    pub fn set_favourite(&mut self, slot: usize, item: ItemType) {
        *self.slots.get_mut(slot).expect("Slot out of range") = Some(item);
    }
}
