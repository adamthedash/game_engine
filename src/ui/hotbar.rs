use egui::{Align2, Color32, Context, Frame, Stroke, Ui, Window};

use super::Drawable;
use crate::item::{ITEMS, ItemId};

#[derive(Default)]
pub struct Hotbar {
    // Each slot holds one item ID
    pub slots: [Option<ItemId>; 10],
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
    pub fn set_favourite(&mut self, slot: usize, item_id: usize) {
        *self.slots.get_mut(slot).expect("Slot out of range") = Some(item_id);
    }
}

impl Drawable for Hotbar {
    fn show_window(&self, ctx: &Context) {
        Window::new("Hotbar")
            .title_bar(false)
            .resizable(false)
            .frame(Frame::window(&ctx.style()).inner_margin(0))
            .anchor(Align2::CENTER_BOTTOM, [0., 0.])
            .show(ctx, |ui| {
                // Remove horizontal padding ebtween slots
                ui.spacing_mut().item_spacing.x = 0.;

                self.show_widget(ui);
            });
    }

    fn show_widget(&self, ui: &mut Ui) {
        let icon_size = 32.;
        let selected_margin_size = 3.;

        let items = ITEMS.get().expect("Item info has not been initialised!");

        ui.columns(self.slots.len(), |columns| {
            // Draw item slots
            columns.iter_mut().enumerate().for_each(|(i, c)| {
                let frame = Frame::new().stroke(Stroke::new(
                    selected_margin_size,
                    // Highlight selected slot
                    if i == self.selected {
                        Color32::LIGHT_BLUE
                    } else {
                        Color32::TRANSPARENT
                    },
                ));

                // Get icon for the item
                let icon = self.slots[i]
                    .and_then(|id| items.get(&id).and_then(|item| item.icon.as_ref()))
                    .unwrap_or_else(|| items.get(&0).unwrap().icon.as_ref().unwrap());

                frame.show(c, |ui| {
                    ui.add(
                        egui::Image::new(icon.clone())
                            .fit_to_exact_size([icon_size, icon_size].into()),
                    );
                });
            });
        });
    }
}
