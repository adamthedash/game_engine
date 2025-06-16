use crate::{
    camera::Camera,
    inventory::{Hotbar, Inventory},
};

/// Information about the player
pub struct Player {
    pub camera: Camera,
    pub arm_length: f32,
    pub inventory: Inventory,
    pub hotbar: Hotbar,
}

impl Player {}
