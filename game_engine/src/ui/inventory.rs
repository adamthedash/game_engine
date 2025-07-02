use egui::{Align2, Color32, FontId, Frame, Key, Vec2, Window};
use enum_map::EnumMap;

use super::Drawable;
use crate::{
    data::{item::ItemType, loader::ITEMS},
    event::{MESSAGE_QUEUE, Message},
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
                    let resp = ui.add(
                        egui::Image::new(icon.clone())
                            .fit_to_exact_size([icon_size, icon_size].into()),
                    );
                    let rect = resp.rect;

                    // Draw item count in bottom right
                    let painter = ui.painter();
                    let font_id = FontId::monospace(font_size);
                    let text = painter.layout_no_wrap(format!("{count}"), font_id, Color32::WHITE);

                    let pos = rect.right_bottom() - text.size();
                    painter.galley(pos, text, Color32::WHITE);

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
