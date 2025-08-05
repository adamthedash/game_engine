pub mod chest;
pub mod crafter;

use crate::{
    data::item::ItemType,
    state::blocks::{chest::ChestState, crafter::CrafterState},
    ui::Drawable,
};

#[derive(Debug, Clone)]
pub enum BlockState {
    Chest(ChestState),
    Crafter(CrafterState),
}

pub trait StatefulBlock: Drawable {
    /// What happens when a player right clicks on the block in the world
    fn on_right_click(&mut self) {}
}

impl BlockState {
    fn as_drawable(&self) -> &dyn Drawable {
        use BlockState::*;
        match self {
            Chest(state) => state,
            Crafter(state) => state,
        }
    }

    fn as_stateful_mut(&mut self) -> &mut dyn StatefulBlock {
        use BlockState::*;
        match self {
            Chest(state) => state,
            Crafter(state) => state,
        }
    }

    /// Returns the interface to this block as a Container if it has that capability
    pub fn as_container_mut(&mut self) -> Option<&mut dyn Container> {
        use BlockState::*;
        match self {
            Chest(state) => Some(state),
            Crafter(state) => Some(state),
        }
    }
}

/// Pass-through trait calls to inner values
impl Drawable for BlockState {
    fn show_window(&self, ctx: &egui::Context) {
        self.as_drawable().show_window(ctx);
    }

    fn show_widget(&self, ui: &mut egui::Ui) {
        self.as_drawable().show_widget(ui);
    }
}

/// Pass-through trait calls to inner values
impl StatefulBlock for BlockState {
    fn on_right_click(&mut self) {
        self.as_stateful_mut().on_right_click();
    }
}

/// Functionality for a thing that can store items
/// Use interior mutability for implementations
pub trait Container {
    fn can_accept(&self, _item: ItemType, _count: usize) -> bool {
        true
    }

    fn add_item(&self, item: ItemType, count: usize);
    fn remove_item(&self, item: ItemType, count: usize);
}
