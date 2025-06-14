use std::time::Duration;

use cgmath::MetricSpace;
use itertools::Itertools;
use winit::event::KeyEvent;

use crate::{
    block::Block,
    player::Player,
    world::{BlockType, Chunk, World},
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

    /// Get the block that the player is looking at
    pub fn get_player_target_block(&self) -> Option<Block> {
        let ray = self.player.camera.ray();
        let (player_chunk_pos, _) = self.player.camera.pos.to_block_pos().to_chunk_offset();

        // Always process the player's chunk
        let player_chunk = [self.world.chunks.get(&player_chunk_pos).unwrap()];

        let candidate_chunks = player_chunk_pos
            .chunks_within(3)
            // Don't double-process player's chunk
            .filter(|chunk_pos| chunk_pos.0 != player_chunk_pos.0)
            // Check which chunks are infront of player
            .filter_map(|chunk_pos| {
                chunk_pos
                    .aabb()
                    .to_f32()
                    .intersect_ray(&ray)
                    .map(|d| (d, chunk_pos))
            })
            // And are within the player's reach
            .filter(|(d, _)| *d <= self.player.arm_length)
            // Process chunks in ascending distance from the player
            .sorted_by(|(d1, _), (d2, _)| d1.total_cmp(d2))
            .flat_map(|(_, chunk_pos)| self.world.chunks.get(&chunk_pos));

        let candidate_chunks = player_chunk.into_iter().chain(candidate_chunks);

        candidate_chunks
            .flat_map(|chunk| {
                // Process blocks chunk-by-chunk
                chunk
                    .iter_blocks()
                    // Can't target air
                    .filter(|b| b.block_type != BlockType::Air)
                    // Check which blocks are within arm's length
                    .filter(|b| {
                        b.block_pos
                            .to_world_pos()
                            .0
                            .distance2(self.player.camera.pos.0)
                            <= self.player.arm_length.powi(2) + 3.
                    })
                    // Check which blocks are infront of player
                    .filter_map(|block| {
                        block
                            .aabb()
                            .to_f32()
                            .intersect_ray(&ray)
                            .map(|d| (d, block))
                    })
                    // Find the closest candidate
                    .min_by(|(d1, _), (d2, _)| d1.total_cmp(d2))
                    .map(|(_, block)| block)
            })
            .next()
    }
}
