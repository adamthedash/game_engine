use crate::{
    data::{item::ItemType, recipe::Recipe},
    state::world::BlockPos,
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
