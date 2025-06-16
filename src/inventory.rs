use rustc_hash::FxHashMap;

use crate::item::ItemId;

#[derive(Default)]
pub struct Inventory {
    // How much of each item the player is holding
    pub items: FxHashMap<ItemId, usize>,
}

#[derive(Default)]
pub struct Hotbar {
    // Each slot holds one item ID
    pub slots: [Option<ItemId>; 10],
    // Slot selected
    pub selected: usize,
}

impl Hotbar {
    /// Move the selected hotbar up or down by one
    pub fn scroll(&mut self, up: bool) {
        if up {
            self.selected += 1;
            self.selected %= self.slots.len();
        } else {
            if self.selected == 0 {
                self.selected += self.slots.len();
            }
            self.selected -= 1;
        }
    }

    /// Set a favourite slot to an item
    pub fn set_favourite(&mut self, slot: usize, item_id: usize) {
        *self.slots.get_mut(slot).expect("Slot out of range") = Some(item_id);
    }
}
