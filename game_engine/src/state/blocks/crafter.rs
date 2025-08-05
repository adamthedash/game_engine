use std::{cell::RefCell, time::Duration};

use egui::{Popup, Vec2, Vec2b, Window, scroll_area::ScrollBarVisibility};
use enum_map::EnumMap;

use super::StatefulBlock;
use crate::{
    InteractionMode,
    data::{
        item::ItemType,
        recipe::{RECIPES, Recipe},
    },
    event::{MESSAGE_QUEUE, Message},
    state::{
        blocks::{Container, Tickable},
        world::BlockPos,
    },
    ui::{
        Drawable,
        helpers::{draw_item_grid, draw_progress_bar, draw_recipe},
        inventory::{TransferItemRequestMessage, TransferItemSource},
    },
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

    pub fn set_recipe(&mut self, recipe: &Recipe) {
        self.recipe = Some(recipe.clone())
    }
}

impl Tickable for CrafterState {
    fn tick(&self, duration: &Duration) {
        // Only process when we've got a recipe
        let Some(recipe) = &self.recipe else {
            return;
        };

        // Only process when we've got enough materials
        let have_materials = {
            let inventory = self.inventory.borrow();
            recipe
                .inputs
                .iter()
                .all(|(item, amount)| inventory[*item] >= *amount)
        };
        if !have_materials {
            return;
        }

        // Make some progress on the recipe
        let mut crafting_juice = self.crafting_juice.borrow_mut();
        *crafting_juice += self.juice_per_second * duration.as_secs_f32();

        // Craft the item if we've got enough
        if *crafting_juice >= recipe.crafting_juice_cost {
            *crafting_juice -= recipe.crafting_juice_cost;

            recipe.inputs.iter().for_each(|(&item, &amount)| {
                self.remove_item(item, amount);
            });

            self.add_item(recipe.output.0, recipe.output.1);
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

#[derive(Debug)]
pub struct SetCraftingRecipeMessage {
    pub block: BlockPos,
    pub recipe: Recipe,
}

impl Drawable for CrafterState {
    fn show_window(&self, ctx: &egui::Context) {
        let icon_size = 32.;
        let num_slots = 8;
        let font_size = 15.;

        let window_size = Vec2::new(icon_size, icon_size) * num_slots as f32;

        Window::new("Crafter")
            .resizable(false)
            // Scroll bar for when we have lots of items
            .scroll(Vec2b { x: false, y: true })
            .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
            .default_width(window_size.x)
            .show(ctx, |ui| {
                // Recipe selector
                let recipe_menu = |ui: &mut egui::Ui| {
                    RECIPES.iter().for_each(|recipe| {
                        if draw_recipe(ui, recipe, icon_size, font_size).clicked() {
                            MESSAGE_QUEUE.send(Message::SetCraftingRecipe(
                                SetCraftingRecipeMessage {
                                    block: self.pos.clone(),
                                    recipe: recipe.clone(),
                                },
                            ));
                        }
                    });
                };

                if let Some(recipe) = &self.recipe {
                    // Click existing recipe to change
                    let resp = draw_recipe(ui, recipe, icon_size, font_size);
                    Popup::menu(&resp).show(recipe_menu);
                } else {
                    // Click button to select
                    ui.allocate_ui(Vec2::new(window_size.x, icon_size), |ui| {
                        ui.menu_button("Select Recipe", recipe_menu);
                    });
                }

                ui.separator();

                // Progress Bar
                let progress = if let Some(recipe) = &self.recipe {
                    (*self.crafting_juice.borrow() / recipe.crafting_juice_cost).clamp(0., 1.)
                } else {
                    0.
                };
                draw_progress_bar(ui, window_size.x, font_size, progress);

                // Storage
                draw_item_grid(ui, "crafter", &self.inventory.borrow(), icon_size)
                    .into_iter()
                    // Filter out responses that weren't drawn
                    .filter_map(|(id, resp)| resp.map(|resp| (id, resp)))
                    .for_each(|(id, resp)| {
                        // Detect keypresses
                        if resp.hovered() {
                            use egui::Key::*;
                            // Item transfer
                            if ui.input(|i| i.key_pressed(T)) {
                                MESSAGE_QUEUE.send(Message::TransferItemRequest(
                                    TransferItemRequestMessage {
                                        item: id,
                                        count: 1,
                                        source: TransferItemSource::Block(self.pos.clone()),
                                    },
                                ));
                            }
                        }
                    });
            });
    }
}
