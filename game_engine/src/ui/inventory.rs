use egui::{Align2, Vec2, Vec2b, Window, scroll_area::ScrollBarVisibility};
use egui_taffy::{
    TuiBuilderLogic,
    taffy::{self, AlignItems, prelude::percent},
    tui,
};
use enum_map::EnumMap;

use super::Drawable;
use crate::{
    data::{
        item::ItemType,
        loader::ITEMS,
        recipe::{RECIPES, Recipe},
    },
    event::{MESSAGE_QUEUE, Message},
    ui::Icon,
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
        let items = ITEMS.get().expect("Items info not initialised!");

        Window::new("Inventory")
            .title_bar(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            // Scroll bar for when we have lots of items
            .scroll(Vec2b { x: false, y: true })
            .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
            .default_width(window_size.x)
            .show(ctx, |ui| {
                // Use egui_taffy to create a grid layout
                tui(ui, ui.id().with("inventory"))
                    .reserve_available_width()
                    .style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Row,
                        flex_wrap: taffy::FlexWrap::Wrap,
                        align_items: Some(AlignItems::Start),
                        size: taffy::Size {
                            width: percent(1.),
                            height: percent(1.),
                        },
                        ..Default::default()
                    })
                    .show(|ui| {
                        ui.reuse_style().add(|ui| {
                            // Draw each item icon if we have some
                            self.items.iter().filter(|(_, count)| **count > 0).for_each(
                                |(id, count)| {
                                    // Create and draw the icon
                                    let icon = Icon {
                                        texture: &items[id].texture,
                                        size: icon_size,
                                        count: Some(*count),
                                        font_size: icon_size / 2.,
                                    };
                                    let resp = ui.ui_add(icon);

                                    // Detect keypresses for hotbar assignment
                                    if resp.hovered() {
                                        use egui::Key::*;
                                        [
                                            Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
                                            Num0,
                                        ]
                                        .into_iter()
                                        .enumerate()
                                        .for_each(
                                            |(slot, key)| {
                                                if ui.egui_ui().input(|i| i.key_pressed(key)) {
                                                    MESSAGE_QUEUE.send(Message::ItemFavourited(
                                                        ItemFavouritedMessage { item: id, slot },
                                                    ));
                                                }
                                            },
                                        );
                                    }
                                },
                            );
                        });
                    });
            });
    }
}
