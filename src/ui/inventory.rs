use egui::{Align2, Vec2, Window};
use rustc_hash::FxHashMap;

use super::Drawable;
use crate::item::{ITEMS, ItemId};

#[derive(Default)]
pub struct Inventory {
    // How much of each item the player is holding
    pub items: FxHashMap<ItemId, usize>,
}

impl Drawable for Inventory {
    fn show_window(&self, ctx: &egui::Context) {
        let icon_size = 32.;
        let window_size = Vec2::new(icon_size, icon_size) * 8.;
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

                    ui.add(
                        egui::Image::new(icon.clone())
                            .fit_to_exact_size([icon_size, icon_size].into()),
                    );
                });
            });
    }

    fn show_widget(&self, ui: &mut egui::Ui) {}
}
