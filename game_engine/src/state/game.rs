use std::time::Duration;

use cgmath::{InnerSpace, MetricSpace};
use itertools::Itertools;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};

use crate::{
    InteractionMode,
    block::Block,
    data::{
        block::BlockType,
        item::ItemType,
        loader::{BLOCKS, ITEMS},
    },
    event::{MESSAGE_QUEUE, Message, Subscriber},
    math::ray::RayCollision,
    state::{
        blocks::{Container, StatefulBlock},
        player::Player,
        world::{Chunk, PlaceBlockMessage, World},
    },
    ui::inventory::TransferItemSource,
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
    pub fn tick(&mut self, duration: &Duration) {
        self.generate_chunks();

        self.world.tick(duration);
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
            self.handle_right_click();
        }
    }

    /// Generate chunks around the player
    fn generate_chunks(&mut self) {
        let pre_generate_buffer = 2; // Generate chunks randomly in this range
        let preemptive_chunks = 8; // Generate this many chunks pre-emptively per frame

        let (player_chunk, _) = self.player.position.pos.to_block_pos().to_chunk_offset();
        let player_vision_chunks =
            (self.player.vision_distance as u32).div_ceil(Chunk::CHUNK_SIZE as u32);

        // Get nearby chunks ordered by distance from player
        let mut nearby_chunks = player_chunk
            .chunks_within(player_vision_chunks + 1 + pre_generate_buffer)
            .map(|pos| {
                let distance = (pos.0 - player_chunk.0).magnitude2();
                (pos, distance)
            })
            .collect::<Vec<_>>();
        nearby_chunks.sort_by_key(|(_, distance)| *distance);

        // Split into chunks within vision distance and outside
        let (nearby_chunks, far_chunks) = nearby_chunks
            .split_once(|(_, distance)| *distance > (player_vision_chunks as i32 + 1).pow(2))
            .unwrap_or_else(|| (&nearby_chunks, &[]));

        // Guarantee the generation of chunks the player can see
        let generating_chunks = nearby_chunks
            .iter()
            .chain(
                // Pre-emptively generate some chunks outside of vision
                far_chunks
                    .iter()
                    .filter(|(pos, _)| !self.world.chunks.contains_key(pos))
                    .take(preemptive_chunks),
            )
            .collect::<Vec<_>>();

        generating_chunks.into_iter().for_each(|(pos, _)| {
            self.world.get_or_generate_chunk(pos);
        });
    }

    /// Get the block that the player is looking at
    pub fn get_player_target_block(&self) -> Option<Block> {
        self.get_player_target_block_verbose()
            .map(|(block, _)| block)
    }

    pub fn get_player_target_block_verbose(&self) -> Option<(Block, RayCollision)> {
        let ray = self.player.position.ray();
        let (player_chunk_pos, _) = self.player.position.pos.to_block_pos().to_chunk_offset();

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
                            .distance2(self.player.position.pos.0)
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
        // Break action needs a target
        let Some(target_block) = self.get_player_target_block() else {
            return;
        };

        let blocks = BLOCKS.get().unwrap();

        // Check if player can break the block
        if blocks[target_block.block_type]
            .data
            .hardness
            .is_none_or(|h| h > self.player.get_breaking_strength())
        {
            // Block is too hard
            return;
        }

        // Break block
        MESSAGE_QUEUE.send(Message::BreakBlock(target_block.block_pos));

        // Give an item to the player
        if let Some(item) = blocks[target_block.block_type].data.item_on_break {
            self.player.inventory.borrow_mut().add_item(item, 1);
        }
    }

    fn place_block(&mut self, target_block: &Block, collision: &RayCollision) {
        // If the player doesn't have anything in their hand, don't do anything
        let Some((item, count)) = self.player.hotbar.get_selected_item() else {
            return;
        };
        if count == 0 {
            return;
        }

        // If the held item can't be placed don't do anything
        let items = ITEMS.get().unwrap();
        let Some(new_block_type) = items[item].data.block else {
            return;
        };

        // Make sure the block we're trying to replace is air
        // TODO: This cast might cause issues at some point
        let adjacent_block_pos = &target_block.block_pos + collision.normal.cast().unwrap();
        let block_type = self
            .world
            .get_block_mut(&adjacent_block_pos)
            .unwrap_or_else(|| panic!("Attempt to place block in uninitalised area!"));
        if *block_type != BlockType::Air {
            return;
        }

        // Remove the item from the player's inventory
        self.player.inventory.borrow_mut().remove_item(item, 1);

        // Place the block
        MESSAGE_QUEUE.send(Message::PlaceBlock(PlaceBlockMessage {
            pos: adjacent_block_pos,
            block: new_block_type,
        }));
    }

    fn handle_right_click(&mut self) {
        // Click action needs a target block
        let Some((target_block, collision)) = self.get_player_target_block_verbose() else {
            return;
        };

        let blocks = BLOCKS.get().unwrap();
        if blocks[target_block.block_type].data.interactable {
            // Interact with the block
            let block_state = self
                .world
                .get_block_state_mut(&target_block.block_pos)
                .expect("Block state doesnt exist!");

            block_state.on_right_click();
        } else {
            self.place_block(&target_block, &collision);
        }
    }
}

#[derive(Debug)]
pub struct TransferItemMessage {
    pub source: TransferItemSource,
    pub dest: TransferItemSource,
    pub item: ItemType,
    pub count: usize,
}

impl Subscriber for GameState {
    fn handle_message(&mut self, event: &Message) {
        if let Message::TransferItem(TransferItemMessage {
            source,
            dest,
            item,
            count,
        }) = event
        {
            use TransferItemSource::*;

            // Remove item from source
            let source: &mut dyn Container = match source {
                Inventory => &mut *self.player.inventory.borrow_mut(),
                Block(block_pos) => self
                    .world
                    .get_block_state_mut(block_pos)
                    .expect("Block state doesn't exist!")
                    .as_container_mut()
                    .expect("Attempt to transfer item to non-container block!"),
            };
            source.remove_item(*item, *count);

            // Add item to destination
            let dest: &mut dyn Container = match dest {
                Inventory => &mut *self.player.inventory.borrow_mut(),
                Block(block_pos) => self
                    .world
                    .get_block_state_mut(block_pos)
                    .expect("Block state doesn't exist!")
                    .as_container_mut()
                    .expect("Attempt to transfer item to non-container block!"),
            };
            assert!(
                dest.can_accept(*item, *count),
                "Container cannot accept item!"
            );

            dest.add_item(*item, *count);
        }
    }
}
