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
    fn on_right_click(&mut self, block_pos: &BlockPos) {
        self.as_stateful_mut().on_right_click(block_pos);
    }
}
