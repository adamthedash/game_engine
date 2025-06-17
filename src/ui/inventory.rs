use egui::{Align2, Color32, FontId, Frame, Vec2, Window};
use rustc_hash::FxHashMap;

use super::Drawable;
use crate::item::{ITEMS, ItemId};

#[derive(Default)]
pub struct Inventory {
    // How much of each item the player is holding
    pub items: FxHashMap<ItemId, usize>,
}

impl Inventory {
    pub fn add_item(&mut self, item: ItemId, count: usize) {
        *self.items.entry(item).or_insert(0) += count;
    }

    pub fn remove_item(&mut self, item: ItemId, count: usize) {
        assert!(
            self.items.get(&item).is_some_and(|x| *x >= count),
            "Not enough items!"
        );

        *self.items.get_mut(&item).unwrap() -= count;

        // Remove from inventory if we've ran out
        if *self.items.get(&item).unwrap() == 0 {
            self.items.remove(&item);
        }
    }
}

impl Drawable for Inventory {
    fn show_window(&self, ctx: &egui::Context) {
        let icon_size = 32.;
        let num_slots = 8;
        let font_size = 15.;

        let window_size = Vec2::new(icon_size, icon_size) * num_slots as f32;
        let items = ITEMS.get().expect("Items info not initialised!");

        Window::new("Inventory")
            .title_bar(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .show(ctx, |ui| {
                // Workaround for window size not working
                // https://github.com/emilk/egui/issues/498#issuecomment-1758462225
                ui.set_width(window_size.x);
                ui.set_height(window_size.y);

                self.items.iter().for_each(|(id, count)| {
                    // Get icon for the item
                    let icon = items.get(id).unwrap().icon.as_ref().unwrap();

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
                        let text =
                            painter.layout_no_wrap(format!("{count}"), font_id, Color32::WHITE);

                        let pos = rect.right_bottom() - text.size();
                        painter.galley(pos, text, Color32::WHITE);
                    });
                });
            });
    }

    fn show_widget(&self, ui: &mut egui::Ui) {}
}
