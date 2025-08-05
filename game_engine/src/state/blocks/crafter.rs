use std::{cell::RefCell, time::Duration};

use egui::Vec2;
use enum_map::EnumMap;

use super::StatefulBlock;
use crate::{
    InteractionMode,
    data::{item::ItemType, loader::ITEMS, recipe::Recipe},
    event::{MESSAGE_QUEUE, Message},
    state::{blocks::Container, world::BlockPos},
    ui::Drawable,
};

#[derive(Debug, Clone)]
pub struct CrafterState {
    pos: BlockPos,
    recipe: Option<Recipe>,
    inventory: RefCell<EnumMap<ItemType, usize>>,
    crafting_juice: RefCell<f32>,
    // TODO: Separate static properties from stateful
    juice_per_second: f32,
}

impl CrafterState {
    pub fn new(pos: &BlockPos) -> Self {
        Self {
            pos: pos.clone(),
            recipe: None,
            inventory: RefCell::default(),
            crafting_juice: RefCell::new(0.),
            juice_per_second: 1.,
        }
    }

    pub fn tick(&self, duration: &Duration) {
        let inventory = self.inventory.borrow();
        // If a recipe is set & we've got the ingredients
        if let Some(recipe) = &self.recipe
            && recipe
                .inputs
                .iter()
                .all(|(item, amount)| inventory[*item] >= *amount)
        {
            let mut crafting_juice = self.crafting_juice.borrow_mut();
            *crafting_juice += self.juice_per_second * duration.as_secs_f32();

            if *crafting_juice >= recipe.crafting_juice_cost {
                // Craft the item
                // TODO: RefCell interface for Container
                recipe.inputs.iter().for_each(|(&item, &amount)| {
                    self.remove_item(item, amount);
                });

                self.add_item(recipe.output.0, recipe.output.1);

                *crafting_juice -= recipe.crafting_juice_cost;
            }
        }
    }
}

impl StatefulBlock for CrafterState {
    fn on_right_click(&mut self) {
        // Go into "Interface mode"
        MESSAGE_QUEUE.send(Message::SetInteractionMode(InteractionMode::Block(
            self.pos.clone(),
        )));
    }
}

impl Container for CrafterState {
    fn add_item(&self, item: ItemType, count: usize) {
        self.inventory.borrow_mut()[item] += count;
    }

    fn remove_item(&self, item: ItemType, count: usize) {
        assert!(self.inventory.borrow()[item] >= count, "Not enough items!");

        self.inventory.borrow_mut()[item] -= count;
    }
}

impl Drawable for CrafterState {
    fn show_window(&self, _ctx: &egui::Context) {
        let icon_size = 32.;
        let num_slots = 8;

        let window_size = Vec2::new(icon_size, icon_size) * num_slots as f32;
        let items = ITEMS.get().expect("Items info not initialised!");
    }
}
