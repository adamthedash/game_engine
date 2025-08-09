use hecs::Entity;

use crate::{
    data::{block::BlockType, item::ItemType, recipe::Recipe},
    entity::components::EntityType,
    state::world::{BlockPos, WorldPos},
};

#[derive(Debug)]
pub struct SetCraftingRecipeMessage {
    pub block: BlockPos,
    pub recipe: Recipe,
}

#[derive(Debug)]
pub struct ItemFavouritedMessage {
    pub item: ItemType,
    pub slot: usize,
}

#[derive(Debug, Clone)]
pub enum TransferItemSource {
    Inventory,
    Block(BlockPos),
}

#[derive(Debug)]
pub struct TransferItemRequestMessage {
    pub item: ItemType,
    pub count: usize,
    pub source: TransferItemSource,
}

#[derive(Debug)]
pub struct SpawnEntityMessage {
    pub pos: WorldPos,
    pub entity_type: EntityType,
}

#[derive(Debug)]
pub struct TransferItemMessage {
    pub source: Entity,
    pub dest: Entity,
    pub item: ItemType,
    pub count: usize,
}

#[derive(Debug)]
pub struct BlockChangedMessage {
    pub pos: BlockPos,
    pub prev_block: BlockType,
    pub new_block: BlockType,
}

#[derive(Debug)]
pub struct PlaceBlockMessage {
    pub pos: BlockPos,
    pub block: BlockType,
}
