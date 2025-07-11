use enum_map::EnumMap;

use crate::{
    InteractionMode,
    data::{item::ItemType, recipe::Recipe},
    state::{game::GameState, world::BlockPos},
    world_gen::ChunkGenerator,
};

pub enum BlockState {
    Chest(ChestState),
    Crafter(CrafterState),
}

pub trait StatefulBlock<G: ChunkGenerator> {
    /// What happens when a player right clicks on the block in the world
    fn on_right_click(
        &mut self,
        _interaction_mode: &mut InteractionMode,
        _game: &mut GameState<G>,
        _block_pos: &BlockPos,
    ) {
    }
}

/// Pass-through trait calls to inner values
impl<G: ChunkGenerator> StatefulBlock<G> for BlockState {
    fn on_right_click(
        &mut self,
        interaction_mode: &mut InteractionMode,
        game: &mut GameState<G>,
        block_pos: &BlockPos,
    ) {
        use BlockState::*;
        match self {
            Chest(chest_state) => chest_state.on_right_click(interaction_mode, game, block_pos),
            Crafter(crafter_state) => {
                crafter_state.on_right_click(interaction_mode, game, block_pos)
            }
        }
    }
}

pub struct CrafterState {
    recipe: Recipe,
}

impl<G: ChunkGenerator> StatefulBlock<G> for CrafterState {}

pub struct ChestState {
    items: EnumMap<ItemType, usize>,
}

impl<G: ChunkGenerator> StatefulBlock<G> for ChestState {
    fn on_right_click(
        &mut self,
        interaction_mode: &mut InteractionMode,
        _game: &mut GameState<G>,
        block_pos: &BlockPos,
    ) {
        // Go into "Interface mode"
        *interaction_mode = InteractionMode::Block(block_pos.clone());
    }
}
