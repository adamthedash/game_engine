use rustc_hash::FxHashMap;

use crate::item::ItemId;

#[derive(Default)]
pub struct Inventory {
    // How much of each item the player is holding
    pub items: FxHashMap<ItemId, usize>,
}
