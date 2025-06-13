use std::time::Duration;

use winit::event::KeyEvent;

use crate::{
    player::Player,
    world::{Chunk, World},
};

/// Holds state information about the game independent of the rendering
pub struct GameState {
    pub player: Player,
    pub world: World,
}

impl GameState {
    /// Once-off stuff to do when a new game state is created
    pub fn init(&mut self) {
        self.generate_chunks();
    }

    /// Update the world by a game tick
    pub fn update(&mut self, time_passed: &Duration) {}

    pub fn handle_keypress(&mut self, event: &KeyEvent) {
        // TODO: Only generate chunks if the player has moved
        self.generate_chunks();
    }

    /// Generate chunks around the player
    fn generate_chunks(&mut self) {
        let (player_chunk, _) = self.player.camera.pos.to_block_pos().to_chunk_offset();
        let player_vision_chunks =
            (self.player.camera.zfar as u32).div_ceil(Chunk::CHUNK_SIZE as u32);
        player_chunk
            .chunks_within(player_vision_chunks + 1)
            // Only generate chunks within vision distance of the player
            .for_each(|chunk_pos| {
                self.world.get_or_generate_chunk(&chunk_pos);
            });
    }
}
