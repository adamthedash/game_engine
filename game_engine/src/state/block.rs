use enum_map::EnumMap;

use crate::{
    InteractionMode,
    data::{item::ItemType, recipe::Recipe},
    event::{MESSAGE_QUEUE, Message},
    state::world::BlockPos,
};

#[derive(Debug, Clone)]
pub enum BlockState {
    Chest(ChestState),
    Crafter(CrafterState),
}

pub trait StatefulBlock {
    /// What happens when a player right clicks on the block in the world
    fn on_right_click(&mut self, _block_pos: &BlockPos) {}
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

#[derive(Debug, Clone)]
pub struct CrafterState {
    recipe: Recipe,
}

impl StatefulBlock for CrafterState {}

#[derive(Default, Debug, Clone)]
pub struct ChestState {
    items: EnumMap<ItemType, usize>,
}

impl StatefulBlock for ChestState {
    fn on_right_click(&mut self, block_pos: &BlockPos) {
        // Go into "Interface mode"
        MESSAGE_QUEUE
            .lock()
            .expect("Failed to lock message queue")
            .push_back(Message::SetInteractionMode(InteractionMode::Block(
                block_pos.clone(),
            )));
    }
}
