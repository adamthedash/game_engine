use std::{cell::RefCell, rc::Rc};

use egui::{Align2, Color32, Context, FontId, Frame, Sense, Stroke, Ui, Window};

use super::{Drawable, inventory::Inventory};
use crate::data::{item::ItemType, loader::ITEMS};

pub struct Hotbar {
    // Each slot holds one item ID
    pub slots: [Option<ItemType>; 10],
    // Slot selected
    pub selected: usize,

    // View onto inventory
    pub inventory: Rc<RefCell<Inventory>>,
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

    /// Get the player's selected item & count
    pub fn get_selected_item(&self) -> Option<(ItemType, usize)> {
        let item_id = self.slots[self.selected]?;
        let count = self.inventory.borrow().items[item_id];

        Some((item_id, count))
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
        let font_size = 15.;

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
                let (icon, count) = if let Some(id) = self.slots[i] {
                    let icon = Some(&items[id].texture);

                    let count = self.inventory.borrow().items[id];
                    (icon, count)
                } else {
                    (None, 0)
                };

                frame.show(c, |ui| {
                    if let Some(icon) = icon {
                        // Draw image
                        let rect = ui
                            .add(
                                egui::Image::new(icon.clone())
                                    .fit_to_exact_size([icon_size, icon_size].into()),
                            )
                            .rect;

                        // Draw item count in bottom right
                        if count > 0 {
                            let painter = ui.painter();
                            let font_id = FontId::monospace(font_size);
                            let text =
                                painter.layout_no_wrap(format!("{count}"), font_id, Color32::WHITE);

                            let pos = rect.right_bottom() - text.size();
                            painter.galley(pos, text, Color32::WHITE);
                        }
                    } else {
                        // Blank icon
                        ui.allocate_exact_size([icon_size, icon_size].into(), Sense::empty());
                    }
                });
            });
        });
    }
}
