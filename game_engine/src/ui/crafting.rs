use std::{cell::RefCell, rc::Rc};

use egui::{
    Align, Color32, FontId, Frame, Label, Layout, Sense, Stroke, TextFormat, Vec2, Window, text,
};

use crate::{
    data::loader::ITEMS,
    ui::{Drawable, Icon, inventory::Inventory},
};

pub struct CraftingWindow {
    pub inventory: Rc<RefCell<Inventory>>,
}

impl Drawable for CraftingWindow {
    fn show_window(&self, ctx: &egui::Context) {
        Window::new("Crafting")
            .default_open(false)
            .resizable(false)
            .show(ctx, |ui| {
                // Workaround for window size not working
                // https://github.com/emilk/egui/issues/498#issuecomment-1758462225
                // ui.set_width(window_size.x);
                // ui.set_height(window_size.y);

                self.show_widget(ui);
            });
    }

    fn show_widget(&self, ui: &mut egui::Ui) {
        let icon_size = 32.;
        let font_size = 15.;

        let recipes = self
            .inventory
            .borrow()
            .get_craftable_recipes()
            .collect::<Vec<_>>();

        let items = ITEMS.get().expect("Items info not initialised!");

        recipes.iter().for_each(|r| {
            ui.allocate_ui_with_layout(Vec2::splat(1.), Layout::left_to_right(Align::Max), |ui| {
                let resp = Frame::default()
                    .stroke(Stroke::new(1., Color32::DARK_GRAY))
                    .show(ui, |ui| {
                        // Inputs on the left
                        r.inputs.iter().for_each(|(item, count)| {
                            Icon {
                                texture: &items[*item].texture,
                                size: icon_size,
                                count: Some(*count),
                                font_size,
                            }
                            .draw(ui);
                        });

                        // Space between input & output
                        let mut arrow = text::LayoutJob::default();
                        arrow.append(
                            ">",
                            0.,
                            TextFormat {
                                font_id: FontId::new(icon_size / 2., egui::FontFamily::Monospace),
                                ..Default::default()
                            },
                        );
                        ui.add_sized(Vec2::splat(icon_size), Label::new(arrow));

                        // Outputs on the right
                        let (item, count) = r.output;
                        Icon {
                            texture: &items[item].texture,
                            size: icon_size,
                            count: Some(count),
                            font_size,
                        }
                        .draw(ui);
                    });

                // Make frame clickable
                let resp = resp.response.interact(Sense::click());

                // Craft on click
                if resp.clicked() {
                    self.inventory.borrow_mut().craft_recipe(r);
                }
            });
        });
    }
}
