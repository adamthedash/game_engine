use rustc_hash::FxHashMap;

pub struct Inventory {
    // How much of each item the player is holding
    pub items: FxHashMap<usize, usize>,
}

pub struct Hotbar {
    // Each slot holds one item ID
    pub slots: [Option<usize>; 10],
    pub selected: usize,
}

impl Default for Hotbar {
    fn default() -> Self {
        Self::new()
    }
}

impl Hotbar {
    pub fn new() -> Self {
        Self {
            slots: Default::default(),
            selected: 0,
        }
    }

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
