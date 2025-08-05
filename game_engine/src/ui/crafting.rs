use std::{cell::RefCell, rc::Rc};

use egui::Window;

use crate::ui::{Drawable, helpers::draw_recipe, inventory::Inventory};

pub struct CraftingWindow {
    pub inventory: Rc<RefCell<Inventory>>,
}

impl Drawable for CraftingWindow {
    fn show_window(&self, ctx: &egui::Context) {
        Window::new("Crafting")
            .default_open(false)
            .resizable(false)
            .show(ctx, |ui| {
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

        recipes.iter().for_each(|r| {
            let resp = draw_recipe(ui, r, icon_size, font_size);

            // Craft on click
            if resp.clicked() {
                self.inventory.borrow_mut().craft_recipe(r);
            }
        });
    }
}
