use std::{cell::RefCell, rc::Rc};

use cgmath::{Rad, Vector3};

use crate::{
    data::loader::ITEMS,
    math::{angles_to_vec3, bbox::AABB, ray::Ray},
    state::world::WorldPos,
    ui::{hotbar::Hotbar, inventory::Inventory},
};

#[derive(Debug, Clone)]
pub struct Position {
    pub pos: WorldPos,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}

impl Position {
    /// Get a ray in the direction the camera is looking
    pub fn ray(&self) -> Ray {
        Ray::new(self.pos.0, angles_to_vec3(self.yaw, self.pitch))
    }
}

/// Information about the player
pub struct Player {
    pub position: Position,
    pub vision_distance: f32,
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
    /// Return the bounding box of the player
    pub fn aabb(&self) -> AABB<f32> {
        let height = 1.8;
        let width = 0.8;
        let head_height = 1.5;

        let diff = Vector3::new(width / 2., height / 2., width / 2.);
        let head_diff = Vector3::unit_y() * head_height / 2.;

        AABB::new(
            &(self.position.pos.0 - diff - head_diff),
            &(self.position.pos.0 + diff - head_diff),
        )
    }
}
