use std::cell::RefCell;

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
    state::{blocks::Container, world::BlockPos},
    ui::{
        Drawable, Icon,
        inventory::{TransferItemRequestMessage, TransferItemSource},
    },
};

#[derive(Debug, Clone)]
pub struct ChestState {
    pub pos: BlockPos,
    pub items: RefCell<EnumMap<ItemType, usize>>,
}

impl ChestState {
    pub fn new(pos: &BlockPos) -> Self {
        Self {
            pos: pos.clone(),
            items: RefCell::default(),
        }
    }
}

impl Container for ChestState {
    fn add_item(&self, item: ItemType, count: usize) {
        self.items.borrow_mut()[item] += count;
    }

    fn remove_item(&self, item: ItemType, count: usize) {
        assert!(self.items.borrow()[item] >= count, "Not enough items!");

        self.items.borrow_mut()[item] -= count;
    }
}

impl StatefulBlock for ChestState {
    fn on_right_click(&mut self) {
        // Go into "Interface mode"
        MESSAGE_QUEUE.send(Message::SetInteractionMode(InteractionMode::Block(
            self.pos.clone(),
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
                            self.items
                                .borrow()
                                .iter()
                                .filter(|(_, count)| **count > 0)
                                .for_each(|(id, count)| {
                                    // Create and draw the icon
                                    let icon = Icon {
                                        texture: &items[id].texture,
                                        size: icon_size,
                                        count: Some(*count),
                                        font_size: icon_size / 2.,
                                    };
                                    let resp = ui.ui_add(icon);
                                    //
                                    // Detect keypresses
                                    if resp.hovered() {
                                        use egui::Key::*;
                                        // Item transfer
                                        if ui.egui_ui().input(|i| i.key_pressed(T)) {
                                            MESSAGE_QUEUE.send(Message::TransferItemRequest(
                                                TransferItemRequestMessage {
                                                    item: id,
                                                    count: 1,
                                                    source: TransferItemSource::Block(
                                                        self.pos.clone(),
                                                    ),
                                                },
                                            ));
                                        }
                                    }
                                });
                        });
                    });
            });
    }
}
