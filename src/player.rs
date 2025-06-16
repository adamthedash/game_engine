use crate::{Hotbar, camera::Camera, ui::inventory::Inventory};

/// Information about the player
pub struct Player {
    pub camera: Camera,
    pub arm_length: f32,
    pub inventory: Inventory,
    pub hotbar: Hotbar,
}

impl Player {}
