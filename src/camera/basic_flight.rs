use std::{f32::consts::FRAC_PI_2, time::Duration};

use cgmath::{Angle, ElementWise, InnerSpace, Rad, Vector3};
use winit::{event::KeyEvent, keyboard::PhysicalKey};

use super::{angles_to_vec3, traits::CameraController};
use crate::{
    bbox::AABB,
    camera::Camera,
    world::{BlockPos, BlockType, World},
};

/// Handles user input to adjust camera
pub struct BasicFlightCameraController {
    move_speed: f32,
    turn_speed: f32,
    // Stateful variables
    left_pressed: bool,
    right_pressed: bool,
    up_pressed: bool,
    down_pressed: bool,
    forward_pressed: bool,
    backwards_pressed: bool,
    pub enabled: bool,
}

impl BasicFlightCameraController {
    pub fn new(move_speed: f32, turn_speed: f32) -> Self {
        assert!(move_speed > 0.);
        assert!(turn_speed > 0.);
        Self {
            move_speed,
            turn_speed,
            left_pressed: false,
            right_pressed: false,
            up_pressed: false,
            down_pressed: false,
            forward_pressed: false,
            backwards_pressed: false,
            enabled: true,
        }
    }
}

impl CameraController for BasicFlightCameraController {
    /// Update the movement state based on user key presses
    fn handle_keypress(&mut self, event: &KeyEvent) {
        if let KeyEvent {
            state,
            physical_key: PhysicalKey::Code(key),
            repeat,
            ..
        } = *event
        {
            use winit::keyboard::KeyCode::*;
            // Toggle
            if state.is_pressed() && key == Escape && !repeat {
                self.enabled ^= true;
            }

            // Don't process anything else if we toggled off
            if !self.enabled {
                return;
            }

            // Movement
            match key {
                KeyW => self.forward_pressed = state.is_pressed(),
                KeyS => self.backwards_pressed = state.is_pressed(),
                KeyA => self.left_pressed = state.is_pressed(),
                KeyD => self.right_pressed = state.is_pressed(),
                Space => self.up_pressed = state.is_pressed(),
                KeyZ => self.down_pressed = state.is_pressed(),
                _ => {}
            }
        }
    }

    /// Turn the camera. delta is in normalised screen coordinates -1 to 1
    fn handle_mouse_move(&mut self, axis: u32, delta: f32, camera: &mut Camera) {
        if !self.enabled {
            return;
        }

        match axis {
            // Horizontal
            0 => {
                camera.yaw += Rad(self.turn_speed * delta);
                camera.yaw = camera.yaw.normalize();
            }
            // Camera
            1 => {
                camera.pitch -= Rad(self.turn_speed * delta);
                // camera.pitch = camera.pitch.normalize();
                // Clip just under fully vertical to avoid weirdness
                camera.pitch.0 = camera.pitch.0.clamp(-FRAC_PI_2 * 0.99, FRAC_PI_2 * 0.99);
            }
            _ => {}
        }
    }

    /// Update the camera position
    fn update_camera(&mut self, camera: &mut Camera, world: &World, duration: &Duration) {
        if !self.enabled {
            return;
        }

        // Step 1: figure out the direction vector the player wants to move in
        let forward = angles_to_vec3(camera.yaw, camera.pitch);
        let right = forward.cross(Vector3::unit_y()).normalize();

        let mut movement_vector = Vector3::new(0., 0., 0.);
        match (self.left_pressed, self.right_pressed) {
            (true, false) => {
                movement_vector -= right;
            }
            (false, true) => {
                movement_vector += right;
            }
            _ => {}
        }
        match (self.forward_pressed, self.backwards_pressed) {
            (true, false) => {
                movement_vector += forward;
            }
            (false, true) => {
                movement_vector -= forward;
            }
            _ => {}
        }
        match (self.up_pressed, self.down_pressed) {
            (true, false) => {
                movement_vector.y += 1.;
            }
            (false, true) => {
                movement_vector.y -= 1.;
            }
            _ => {}
        }
        if movement_vector.magnitude2() == 0. {
            return;
        }
        movement_vector = movement_vector.normalize();

        // Step 2: Figure out if we're colliding with any blocks
        let player_block_pos = camera.pos.to_block_pos();
        let player_aabb = AABB::new(
            &camera.pos.0.add_element_wise(-0.4),
            &camera.pos.0.add_element_wise(0.4),
        );

        let colliding_with = |pos: &BlockPos| {
            if let Some(block) = world.get_block(pos) {
                if block.block_type == BlockType::Air {
                    false
                } else {
                    player_aabb.intersects(&block.aabb().to_f32())
                }
            } else {
                false
            }
        };

        if movement_vector.x != 0.
            && colliding_with(&BlockPos(
                player_block_pos.0 + movement_vector.x.signum() as i32 * Vector3::unit_x(),
            ))
        {
            movement_vector.x = 0.;
        }
        if movement_vector.y != 0.
            && colliding_with(&BlockPos(
                player_block_pos.0 + movement_vector.y.signum() as i32 * Vector3::unit_y(),
            ))
        {
            movement_vector.y = 0.;
        }
        if movement_vector.z != 0.
            && colliding_with(&BlockPos(
                player_block_pos.0 + movement_vector.z.signum() as i32 * Vector3::unit_z(),
            ))
        {
            movement_vector.z = 0.;
        }

        // Apply the movement vector
        camera.pos.0 += movement_vector * self.move_speed * duration.as_secs_f32();
    }
}
