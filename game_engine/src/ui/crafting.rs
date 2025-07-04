use std::{cell::RefCell, rc::Rc};

use egui::{Align, Color32, Frame, Layout, Sense, Stroke, Vec2, Window};

use crate::{
    data::loader::ITEMS,
    ui::{Drawable, draw_icon, inventory::Inventory},
};

pub struct CraftingWindow {
    pub inventory: Rc<RefCell<Inventory>>,
}

impl Drawable for CraftingWindow {
    fn show_window(&self, ctx: &egui::Context) {
        Window::new("Crafting")
            .title_bar(false)
            .resizable(false)
            //.anchor(Align2::CENTER_CENTER, [0., 0.])
            .show(ctx, |ui| {
                // Workaround for window size not working
                // https://github.com/emilk/egui/issues/498#issuecomment-1758462225
                //ui.set_width(window_size.x);
                //ui.set_height(window_size.y);

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
                        let mut resps = r
                            .inputs
                            .iter()
                            .map(|(item, count)| {
                                let icon = &items[*item].texture;
                                draw_icon(ui, icon, Some(*count), icon_size, font_size)
                            })
                            .collect::<Vec<_>>();

                        // Space between input & output
                        let (_, resp) =
                            ui.allocate_exact_size([icon_size, icon_size].into(), Sense::empty());
                        resps.push(resp);

                        // Outputs on the right
                        let (item, count) = r.output;
                        let icon = &items[item].texture;
                        let resp = draw_icon(ui, icon, Some(count), icon_size, font_size);
                        resps.push(resp);
                        resps
                    });

                // Craft on click
                if resp.inner.iter().any(|r| r.clicked()) {
                    self.inventory.borrow_mut().craft_recipe(r);
                }
            });
        });
    }
}
