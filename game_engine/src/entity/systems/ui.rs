use egui::{
    Align2, Color32, Frame, Popup, Sense, Stroke, Vec2, Vec2b, Window,
    scroll_area::ScrollBarVisibility,
};
use hecs::EntityRef;

use crate::{
    data::{loader::ITEMS, recipe::RECIPES},
    entity::components::{Container, Crafter, Hotbar, UIType},
    event::{
        MESSAGE_QUEUE, Message,
        messages::{
            ItemFavouritedMessage, SetCraftingRecipeMessage, TransferItemRequestMessage,
            TransferItemSource,
        },
    },
    state::world::BlockPos,
    ui::{
        Icon,
        helpers::{draw_item_grid, draw_progress_bar, draw_recipe},
    },
};

pub fn draw_ui(ctx: &egui::Context, entity: EntityRef<'_>) {
    let ui_type = entity
        .get::<&UIType>()
        .expect("Entity has no UI component!");

    match *ui_type {
        UIType::Chest => draw_chest(ctx, entity),
        UIType::Crafter => draw_crafter(ctx, entity),
    }
}

pub fn draw_chest(ctx: &egui::Context, entity: EntityRef<'_>) {
    let mut entity = entity.query::<(&BlockPos, &Container)>();
    let (block_pos, container) = entity
        .get()
        .expect("Chest doesn't have the right components!");

    let icon_size = 32.;
    let num_slots = 8;

    let window_size = Vec2::new(icon_size, icon_size) * num_slots as f32;

    Window::new("Chest")
        .resizable(false)
        // Scroll bar for when we have lots of items
        .scroll(Vec2b { x: false, y: true })
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
        .default_width(window_size.x)
        .show(ctx, |ui| {
            draw_item_grid(ui, "chest", &container.items, icon_size)
                .into_iter()
                // Filter out responses that weren't drawn
                .filter_map(|(id, resp)| resp.map(|resp| (id, resp)))
                .for_each(|(id, resp)| {
                    // Detect keypresses
                    if resp.hovered() {
                        use egui::Key::*;
                        // Item transfer
                        if ui.input(|i| i.key_pressed(T)) {
                            MESSAGE_QUEUE.send(Message::TransferItemRequest(
                                TransferItemRequestMessage {
                                    item: id,
                                    count: 1,
                                    source: TransferItemSource::Block(block_pos.clone()),
                                },
                            ));
                        }
                    }
                });
        });
}

pub fn draw_crafter(ctx: &egui::Context, entity: EntityRef<'_>) {
    let mut entity = entity.query::<(&BlockPos, &Container, &Crafter)>();
    let (block_pos, container, crafter) = entity
        .get()
        .expect("Chest doesn't have the right components!");

    let icon_size = 32.;
    let num_slots = 8;
    let font_size = 15.;

    let window_size = Vec2::new(icon_size, icon_size) * num_slots as f32;

    Window::new("Crafter")
        .resizable(false)
        // Scroll bar for when we have lots of items
        .scroll(Vec2b { x: false, y: true })
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
        .default_width(window_size.x)
        .show(ctx, |ui| {
            // Recipe selector
            let recipe_menu = |ui: &mut egui::Ui| {
                RECIPES.iter().for_each(|recipe| {
                    if draw_recipe(ui, recipe, icon_size, font_size).clicked() {
                        MESSAGE_QUEUE.send(Message::SetCraftingRecipe(SetCraftingRecipeMessage {
                            block: block_pos.clone(),
                            recipe: recipe.clone(),
                        }));
                    }
                });
            };

            if let Some(recipe) = &crafter.recipe {
                // Click existing recipe to change
                let resp = draw_recipe(ui, recipe, icon_size, font_size);
                Popup::menu(&resp).show(recipe_menu);
            } else {
                // Click button to select
                ui.allocate_ui(Vec2::new(window_size.x, icon_size), |ui| {
                    ui.menu_button("Select Recipe", recipe_menu);
                });
            }

            ui.separator();

            // Progress Bar
            let progress = if let Some(recipe) = &crafter.recipe {
                (crafter.crafting_juice / recipe.crafting_juice_cost).clamp(0., 1.)
            } else {
                0.
            };
            draw_progress_bar(ui, window_size.x, font_size, progress);

            // Storage
            draw_item_grid(ui, "crafter", &container.items, icon_size)
                .into_iter()
                // Filter out responses that weren't drawn
                .filter_map(|(id, resp)| resp.map(|resp| (id, resp)))
                .for_each(|(id, resp)| {
                    // Detect keypresses
                    if resp.hovered() {
                        use egui::Key::*;
                        // Item transfer
                        if ui.input(|i| i.key_pressed(T)) {
                            MESSAGE_QUEUE.send(Message::TransferItemRequest(
                                TransferItemRequestMessage {
                                    item: id,
                                    count: 1,
                                    source: TransferItemSource::Block(block_pos.clone()),
                                },
                            ));
                        }
                    }
                });
        });
}

/// Displayer the inventory for a given entity
pub fn draw_inventory(ctx: &egui::Context, entity: EntityRef<'_>) {
    let inventory = entity
        .get::<&Container>()
        .expect("Failed to get container for entity");

    let icon_size = 32.;
    let num_slots = 8;

    let window_size = Vec2::new(icon_size, icon_size) * num_slots as f32;

    Window::new("Inventory")
        .resizable(false)
        // Scroll bar for when we have lots of items
        .scroll(Vec2b { x: false, y: true })
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
        .default_width(window_size.x)
        .show(ctx, |ui| {
            draw_item_grid(ui, "inventory", &inventory.items, icon_size)
                .into_iter()
                // Filter out responses that weren't drawn
                .filter_map(|(id, resp)| resp.map(|resp| (id, resp)))
                .for_each(|(id, resp)| {
                    // Detect keypresses
                    if resp.hovered() {
                        use egui::Key::*;
                        // Hotbar assignment
                        [Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9, Num0]
                            .into_iter()
                            .enumerate()
                            .for_each(|(slot, key)| {
                                if ui.input(|i| i.key_pressed(key)) {
                                    MESSAGE_QUEUE.send(Message::ItemFavourited(
                                        ItemFavouritedMessage { item: id, slot },
                                    ));
                                }
                            });

                        // Item transfer
                        if ui.input(|i| i.key_pressed(T)) {
                            MESSAGE_QUEUE.send(Message::TransferItemRequest(
                                TransferItemRequestMessage {
                                    item: id,
                                    count: 1,
                                    source: TransferItemSource::Inventory,
                                },
                            ));
                        }
                    }
                });
        });
}

pub fn draw_crafting_window(ctx: &egui::Context, entity: EntityRef<'_>) {
    let mut inventory = entity
        .get::<&mut Container>()
        .expect("Failed to get container for entity");

    let icon_size = 32.;
    let font_size = 15.;

    let recipes = inventory.get_craftable_recipes().collect::<Vec<_>>();

    Window::new("Crafting")
        .default_open(false)
        .resizable(false)
        .show(ctx, |ui| {
            recipes.iter().for_each(|r| {
                let resp = draw_recipe(ui, r, icon_size, font_size);

                // Craft on click
                if resp.clicked() {
                    inventory.craft_recipe(r);
                }
            });
        });
}

pub fn draw_hotbar(ctx: &egui::Context, entity: EntityRef<'_>) {
    let mut query = entity.query::<(&Container, &Hotbar)>();
    let (inventory, hotbar) = query.get().unwrap();

    let icon_size = 32.;
    let selected_margin_size = 3.;
    let font_size = 15.;

    let items = ITEMS.get().expect("Item info has not been initialised!");

    Window::new("Hotbar")
        .title_bar(false)
        .resizable(false)
        .frame(Frame::window(&ctx.style()).inner_margin(0))
        .anchor(Align2::CENTER_BOTTOM, [0., 0.])
        .show(ctx, |ui| {
            // Remove horizontal padding ebtween slots
            ui.spacing_mut().item_spacing.x = 0.;

            ui.columns(hotbar.slots.len(), |columns| {
                // Draw item slots
                columns.iter_mut().enumerate().for_each(|(i, c)| {
                    let frame = Frame::new().stroke(Stroke::new(
                        selected_margin_size,
                        // Highlight selected slot
                        if i == hotbar.selected {
                            Color32::LIGHT_BLUE
                        } else {
                            Color32::TRANSPARENT
                        },
                    ));

                    // Get icon for the item
                    let (icon, count) = if let Some(id) = hotbar.slots[i] {
                        let icon = Some(&items[id].texture);

                        let count = inventory.items[id];
                        (icon, count)
                    } else {
                        (None, 0)
                    };

                    frame.show(c, |ui| {
                        if let Some(icon) = icon {
                            Icon {
                                texture: icon,
                                size: icon_size,
                                count: if count > 0 { Some(count) } else { None },
                                font_size,
                            }
                            .draw(ui);
                        } else {
                            // Blank icon
                            ui.allocate_exact_size([icon_size, icon_size].into(), Sense::empty());
                        }
                    });
                });
            });
        });
}
