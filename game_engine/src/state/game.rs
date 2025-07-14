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
    event::{MESSAGE_QUEUE, Message},
    math::ray::RayCollision,
    state::{
        block::StatefulBlock,
        player::Player,
        world::{BlockChangedMessage, BlockPos, Chunk, World},
    },
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
    pub fn update(&mut self, _time_passed: &Duration) {
        self.generate_chunks();
    }

    pub fn handle_keypress(&mut self, _event: &KeyEvent) {}

    pub fn handle_mouse_key(&mut self, event: &WindowEvent, mode: &mut InteractionMode) {
        assert!(matches!(event, WindowEvent::MouseInput { .. }));

        if !matches!(mode, InteractionMode::Game) {
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
            self.handle_right_click(mode);
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
            .map(|(block, _)| block)
    }

    pub fn get_player_target_block_verbose(&self) -> Option<(Block, RayCollision)> {
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
                    .map(|col| (chunk_pos, col))
            })
            // And are within the player's reach
            .filter(|(_, col)| col.distance <= self.player.arm_length)
            // Process chunks in ascending distance from the player
            .sorted_by(|(_, c1), (_, c2)| c1.distance.total_cmp(&c2.distance))
            .flat_map(|(chunk_pos, _)| self.world.chunks.get(&chunk_pos));

        let candidate_chunks = player_chunk.into_iter().chain(candidate_chunks);

        let blocks = BLOCKS.get().expect("Block data not initalised!");
        candidate_chunks
            .flat_map(|chunk| {
                // Process blocks chunk-by-chunk
                chunk
                    .iter_blocks()
                    // Can't target air
                    .filter(|b| blocks[b.block_type].data.renderable)
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
                            .map(|col| (block, col))
                    })
                    // Find the closest candidate
                    .sorted_by(|(_, c1), (_, c2)| c1.distance.total_cmp(&c2.distance))
            })
            .next()
    }

    /// Attempt to break the block the player is targeting
    fn break_block(&mut self) {
        if let Some(target_block) = self.get_player_target_block() {
            let blocks = BLOCKS.get().unwrap();

            // Check if player can break the block
            let block_type = self.world.get_block_mut(&target_block.block_pos).unwrap();
            if blocks[*block_type]
                .data
                .hardness
                .is_none_or(|h| h > self.player.get_breaking_strength())
            {
                // Block is too hard
                return;
            }

            // Break block
            let old_block = std::mem::replace(block_type, BlockType::Air);

            // Remove the block state if it was stateful
            if blocks[old_block].data.state.is_some() {
                let old_state = self.world.block_states.remove(&target_block.block_pos);
                assert!(
                    old_state.is_some(),
                    "Attempted to remove stateful block, but no state existed! {:?}",
                    target_block.block_pos
                );
            }

            // Give an item to the player
            let item = blocks[old_block].data.item_on_break;
            self.player.inventory.borrow_mut().add_item(item, 1);

            // Tell the world a block has changed
            MESSAGE_QUEUE
                .lock()
                .unwrap()
                .push_back(Message::BlockChanged(BlockChangedMessage {
                    pos: target_block.block_pos,
                    prev_block: old_block,
                    new_block: BlockType::Air,
                }));
        }
    }

    fn handle_right_click(&mut self, mode: &mut InteractionMode) {
        let items = ITEMS.get().unwrap();
        let blocks = BLOCKS.get().unwrap();

        if let Some((target_block, collision)) = self.get_player_target_block_verbose() {
            if blocks[target_block.block_type].data.interactable {
                // Interact with the block
                let block_state = self
                    .world
                    .get_block_state_mut(&target_block.block_pos)
                    .expect("Block state doesnt exist!");

                block_state.on_right_click(&target_block.block_pos);
            } else {
                // Attempt to place block
                if let Some((item, count)) = self.player.hotbar.get_selected_item()
                    && count > 0
                    && let Some(new_block_type) = items[item].data.block
                {
                    // Get the adjacent block
                    // TODO: This cast might cause issues at some point
                    let adjacent_block_pos =
                        BlockPos(target_block.block_pos.0 + collision.normal.cast().unwrap());

                    if let Some(block_type) = self.world.get_block_mut(&adjacent_block_pos)
                    // Only place in air blocks
                    && *block_type == BlockType::Air
                    {
                        // Place the block
                        *block_type = new_block_type;
                        // Create a state if the block is stateful
                        if let Some(state_fn) = blocks[new_block_type].data.state {
                            let old_state = self
                                .world
                                .block_states
                                .insert(adjacent_block_pos.clone(), state_fn());
                            assert!(
                                old_state.is_none(),
                                "Overwrote existing state at {adjacent_block_pos:?}! {old_state:?} -> {new_block_type:?}"
                            );
                        }

                        // Remove the item from the player's inventory
                        self.player.inventory.borrow_mut().remove_item(item, 1);

                        // Tell the world a block has changed
                        MESSAGE_QUEUE
                            .lock()
                            .unwrap()
                            .push_back(Message::BlockChanged(BlockChangedMessage {
                                pos: adjacent_block_pos,
                                prev_block: BlockType::Air,
                                new_block: new_block_type,
                            }));
                    }
                }
            }
        }
    }
}

/// Convert a vector offset to it's closest unit offset along one cardinal direction
#[inline]
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
