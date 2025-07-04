use egui::{Align2, Frame, Key, Vec2, Window};
use enum_map::EnumMap;

use super::Drawable;
use crate::{
    data::{
        item::ItemType,
        loader::ITEMS,
        recipe::{RECIPES, Recipe},
    },
    event::{MESSAGE_QUEUE, Message},
    ui::draw_icon,
};

#[derive(Default)]
pub struct Inventory {
    // How much of each item the player is holding
    pub items: EnumMap<ItemType, usize>,
}

impl Inventory {
    pub fn add_item(&mut self, item: ItemType, count: usize) {
        self.items[item] += count;
    }

    pub fn remove_item(&mut self, item: ItemType, count: usize) {
        assert!(self.items[item] >= count, "Not enough items!");

        self.items[item] -= count;
    }

    /// Get the recipes the player can currently craft based on what they have on them
    pub fn get_craftable_recipes(&self) -> impl Iterator<Item = &'static Recipe> {
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

#[derive(Debug)]
pub struct ItemFavouritedMessage {
    pub item: ItemType,
    pub slot: usize,
}

impl Drawable for Inventory {
    fn show_window(&self, ctx: &egui::Context) {
        let icon_size = 32.;
        let num_slots = 8;

        let window_size = Vec2::new(icon_size, icon_size) * num_slots as f32;

        Window::new("Inventory")
            .title_bar(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .show(ctx, |ui| {
                // Workaround for window size not working
                // https://github.com/emilk/egui/issues/498#issuecomment-1758462225
                ui.set_width(window_size.x);
                ui.set_height(window_size.y);

                self.show_widget(ui);
            });
    }

    fn show_widget(&self, ui: &mut egui::Ui) {
        let font_size = 15.;
        let icon_size = 32.;

        let items = ITEMS.get().expect("Items info not initialised!");

        self.items
            .iter()
            .filter(|(_, count)| **count > 0)
            .for_each(|(id, count)| {
                // Get icon for the item
                let icon = &items[id].texture;

                let frame = Frame::NONE;
                frame.show(ui, |ui| {
                    let resp = draw_icon(ui, icon, Some(*count), icon_size, font_size);

                    // Hotbar assignment
                    if resp.hovered() {
                        use Key::*;
                        [Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9, Num0]
                            .into_iter()
                            .enumerate()
                            .for_each(|(slot, key)| {
                                if ui.input(|i| i.key_pressed(key)) {
                                    MESSAGE_QUEUE
                                        .lock()
                                        .expect("Failed to lock message queue")
                                        .push_back(Message::ItemFavourited(
                                            ItemFavouritedMessage { item: id, slot },
                                        ));
                                }
                            });
                    }
                });
            });
    }
}
