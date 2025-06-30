use std::{cell::RefCell, rc::Rc};

use crate::{
    camera::Camera,
    ui::{hotbar::Hotbar, inventory::Inventory},
};

/// Information about the player
pub struct Player {
    pub camera: Camera,
    pub arm_length: f32,
    pub inventory: Rc<RefCell<Inventory>>,
    pub hotbar: Hotbar,
}
