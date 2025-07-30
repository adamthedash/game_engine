use egui::{Vec2, Vec2b, Window, scroll_area::ScrollBarVisibility};
use egui_taffy::{
    TuiBuilderLogic,
    taffy::{self, AlignItems, prelude::percent},
    tui,
};
use enum_map::EnumMap;

use super::StatefulBlock;
use crate::{
    InteractionMode,
    data::{item::ItemType, loader::ITEMS},
    event::{MESSAGE_QUEUE, Message},
    state::world::BlockPos,
    ui::{Drawable, Icon},
};

#[derive(Default, Debug, Clone)]
pub struct ChestState {
    pub items: EnumMap<ItemType, usize>,
}

impl StatefulBlock for ChestState {
    fn on_right_click(&mut self, block_pos: &BlockPos) {
        // Go into "Interface mode"
        MESSAGE_QUEUE.send(Message::SetInteractionMode(InteractionMode::Block(
            block_pos.clone(),
        )));
    }
}

impl Drawable for ChestState {
    fn show_window(&self, ctx: &egui::Context) {
        let icon_size = 32.;
        let num_slots = 8;

        let window_size = Vec2::new(icon_size, icon_size) * num_slots as f32;
        let items = ITEMS.get().expect("Items info not initialised!");

        Window::new("Chest")
            .resizable(false)
            // Scroll bar for when we have lots of items
            .scroll(Vec2b { x: false, y: true })
            .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
            .default_width(window_size.x)
            .show(ctx, |ui| {
                // Use egui_taffy to create a grid layout
                tui(ui, ui.id().with("chest"))
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
                                },
                            );
                        });
                    });
            });
    }
}
