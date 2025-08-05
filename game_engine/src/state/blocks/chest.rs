use std::cell::RefCell;

use egui::{Vec2, Vec2b, Window, scroll_area::ScrollBarVisibility};
use enum_map::EnumMap;

use super::StatefulBlock;
use crate::{
    InteractionMode,
    data::item::ItemType,
    event::{MESSAGE_QUEUE, Message},
    state::{blocks::Container, world::BlockPos},
    ui::{
        Drawable,
        helpers::draw_item_grid,
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

        Window::new("Chest")
            .resizable(false)
            // Scroll bar for when we have lots of items
            .scroll(Vec2b { x: false, y: true })
            .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
            .default_width(window_size.x)
            .show(ctx, |ui| {
                draw_item_grid(ui, "crafter", &self.items.borrow(), icon_size)
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
                                        source: TransferItemSource::Block(self.pos.clone()),
                                    },
                                ));
                            }
                        }
                    });
            });
    }
}
