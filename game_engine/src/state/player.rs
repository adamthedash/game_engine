use std::{cell::RefCell, rc::Rc};

use crate::{
    camera::Camera,
    data::loader::ITEMS,
    ui::{hotbar::Hotbar, inventory::Inventory},
};

/// Information about the player
pub struct Player {
    pub camera: Camera,
    pub arm_length: f32,
    pub inventory: Rc<RefCell<Inventory>>,
    pub hotbar: Hotbar,
}

impl Player {
    pub fn get_breaking_strength(&self) -> u32 {
        let player_item = self.hotbar.get_selected_item();

        if let Some((item, count)) = player_item
            && count > 0
        {
            ITEMS.get().unwrap()[item]
                .data
                .breaking_strength
                .unwrap_or(0)
        } else {
            0
        }
    }
}
