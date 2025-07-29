pub mod chest;
pub mod crafter;

use crate::{
    state::{
        blocks::{chest::ChestState, crafter::CrafterState},
        world::BlockPos,
    },
    ui::Drawable,
};

#[derive(Debug, Clone)]
pub enum BlockState {
    Chest(ChestState),
    Crafter(CrafterState),
}

pub trait StatefulBlock: Drawable {
    /// What happens when a player right clicks on the block in the world
    fn on_right_click(&mut self, _block_pos: &BlockPos) {}
}

/// Pass-through trait calls to inner values
impl Drawable for BlockState {
    fn show_window(&self, ctx: &egui::Context) {
        use BlockState::*;
        match self {
            Chest(chest_state) => chest_state.show_window(ctx),
            Crafter(crafter_state) => crafter_state.show_window(ctx),
        }
    }

    fn show_widget(&self, ui: &mut egui::Ui) {
        use BlockState::*;
        match self {
            Chest(chest_state) => chest_state.show_widget(ui),
            Crafter(crafter_state) => crafter_state.show_widget(ui),
        }
    }
}

/// Pass-through trait calls to inner values
impl StatefulBlock for BlockState {
    fn on_right_click(&mut self, block_pos: &BlockPos) {
        use BlockState::*;
        match self {
            Chest(chest_state) => chest_state.on_right_click(block_pos),
            Crafter(crafter_state) => crafter_state.on_right_click(block_pos),
        }
    }
}
