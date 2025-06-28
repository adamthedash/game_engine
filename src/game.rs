use std::time::Duration;

use cgmath::{InnerSpace, MetricSpace, Vector3, Zero};
use itertools::Itertools;
use rand::random_bool;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};

use crate::{
    InteractionMode,
    block::Block,
    data::{
        block::BlockType,
        loader::{BLOCKS, ITEMS},
    },
    player::Player,
    world::{BlockPos, Chunk, World, WorldPos},
    world_gen::ChunkGenerator,
};

/// Holds state information about the game independent of the rendering
pub struct GameState<G: ChunkGenerator> {
    pub player: Player,
    pub world: World<G>,
}

impl<G: ChunkGenerator> GameState<G> {
    /// Once-off stuff to do when a new game state is created
    pub fn init(&mut self) {
        self.generate_chunks();
    }

    /// Update the world by a game tick
    pub fn update(&mut self, time_passed: &Duration) {
        self.generate_chunks();
    }

    pub fn handle_keypress(&mut self, event: &KeyEvent) {}

    pub fn handle_mouse_key(&mut self, event: &WindowEvent, mode: &InteractionMode) {
        assert!(matches!(event, WindowEvent::MouseInput { .. }));

        if *mode != InteractionMode::Game {
            return;
        }

        if matches!(
            event,
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            }
        ) {
            self.break_block();
        } else if matches!(
            event,
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            }
        ) {
            self.place_block();
        }
    }

    /// Generate chunks around the player
    fn generate_chunks(&mut self) {
        let pre_generate_buffer = 2; // Generate chunks randomly in this range
        let pre_generate_chance = 0.02;

        let (player_chunk, _) = self
            .player
            .camera
            .pos
            .get()
            .to_block_pos()
            .to_chunk_offset();
        let player_vision_chunks =
            (self.player.camera.zfar.get() as u32).div_ceil(Chunk::CHUNK_SIZE as u32);
        player_chunk
            .chunks_within(player_vision_chunks + 1 + pre_generate_buffer)
            // Only generate chunks within vision distance of the player
            .for_each(|chunk_pos| {
                let generate = if (chunk_pos.0 - player_chunk.0).magnitude2()
                    > (player_vision_chunks as i32 + 1).pow(2)
                {
                    // Random gen area
                    random_bool(pre_generate_chance)
                } else {
                    // Guarantee generation
                    true
                };

                if generate {
                    self.world.get_or_generate_chunk(&chunk_pos);
                }
            });
    }

    /// Get the block that the player is looking at
    pub fn get_player_target_block(&self) -> Option<Block> {
        self.get_player_target_block_verbose()
            .map(|(_, _, block)| block)
    }

    pub fn get_player_target_block_verbose(&self) -> Option<(f32, WorldPos, Block)> {
        let ray = self.player.camera.ray();
        let (player_chunk_pos, _) = self
            .player
            .camera
            .pos
            .get()
            .to_block_pos()
            .to_chunk_offset();

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
                    .filter(|b| b.block_type == BlockType::Air)
                    // Check which blocks are within arm's length
                    .filter(|b| {
                        b.block_pos
                            .to_world_pos()
                            .0
                            .distance2(self.player.camera.pos.get().0)
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
                    // Compute the intersection point
                    .map(|(d, block)| {
                        let intersect = WorldPos(ray.pos + ray.direction * d);
                        (d, intersect, block)
                    })
            })
            .next()
    }

    /// Attempt to break the block the player is targeting
    fn break_block(&mut self) {
        if let Some(target_block) = self.get_player_target_block() {
            // Break block
            let old_block = std::mem::replace(
                self.world.get_block_mut(&target_block.block_pos).unwrap(),
                BlockType::Air,
            );

            // Give an item to the player
            let item = BLOCKS.get().unwrap()[old_block].item_on_break;
            self.player.inventory.borrow_mut().add_item(item, 1);

            // Update block exposure information
            // TODO: Change to block-level updates instead of chunk level
            let (chunk_pos, _) = target_block.block_pos.to_chunk_offset();
            self.world.update_exposed_blocks(&chunk_pos);
        }
    }

    /// Attempt the place a block where the player is looking
    fn place_block(&mut self) {
        if let Some((id, count)) = self.player.hotbar.get_selected_item() {
            assert!(count > 0);

            // Check if item is placeable
            if let Some(new_block_type) = ITEMS.get().unwrap()[id].block
                && let Some((_, intersect, target_block)) = self.get_player_target_block_verbose()
            {
                // Get the adjacent block
                let direction_vector = intersect.0 - target_block.block_pos.centre().0;
                let offset = to_cardinal_offset(&direction_vector);
                let adjacent_block_pos = BlockPos(target_block.block_pos.0 + offset);

                if let Some(block_type) = self.world.get_block_mut(&adjacent_block_pos)
                    // Only place in air blocks
                    && *block_type == BlockType::Air
                {
                    // TODO: move this selection logic elsewhere
                    *block_type = new_block_type;
                    self.player.inventory.borrow_mut().remove_item(id, 1);

                    // Update block exposure information
                    // TODO: Change to block-level updates instead of chunk level
                    let (chunk_pos, _) = target_block.block_pos.to_chunk_offset();
                    self.world.update_exposed_blocks(&chunk_pos);
                }
            }
        }
    }
}

/// Convert a vector offset to it's closest unit offset along one cardinal direction
fn to_cardinal_offset(vec: &Vector3<f32>) -> Vector3<i32> {
    // Get the axis with the largest magnitude
    let largest_mag = [vec[0], vec[1], vec[2]]
        .into_iter()
        .enumerate()
        .max_by(|(_, x1), (_, x2)| x1.abs().total_cmp(&x2.abs()))
        .map(|(i, _)| i)
        .unwrap();

    // Get the unit vector along this axis
    let mut offset = Vector3::zero();
    offset[largest_mag] = if vec[largest_mag] > 0. { 1 } else { -1 };

    offset
}
