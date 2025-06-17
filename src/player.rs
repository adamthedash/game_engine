use std::{cell::RefCell, rc::Rc};

use crate::{Hotbar, camera::Camera, ui::inventory::Inventory};

/// Information about the player
pub struct Player {
    pub camera: Camera,
    pub arm_length: f32,
    pub inventory: Rc<RefCell<Inventory>>,
    pub hotbar: Hotbar,
}
