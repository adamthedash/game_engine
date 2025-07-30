use std::{f32::consts::FRAC_PI_2, time::Duration};

use cgmath::{Angle, InnerSpace, Rad, Vector3, Zero};
use winit::{event::KeyEvent, keyboard::PhysicalKey};

use super::traits::PlayerController;
use crate::{
    camera::collision::predict_collisions,
    math::angles_to_vec3,
    state::{
        player::{Player, Position},
        world::World,
    },
};

/// Handles user input to adjust camera
pub struct BasicFlightController {
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

impl BasicFlightController {
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

impl PlayerController for BasicFlightController {
    fn toggle(&mut self) {
        if self.enabled {
            // Un-press any buttons
            self.left_pressed = false;
            self.right_pressed = false;
            self.up_pressed = false;
            self.down_pressed = false;
            self.forward_pressed = false;
            self.backwards_pressed = false;
        }
        self.enabled ^= true;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    /// Update the movement state based on user key presses
    fn handle_keypress(&mut self, event: &KeyEvent) {
        if let KeyEvent {
            state,
            physical_key: PhysicalKey::Code(key),
            ..
        } = *event
        {
            use winit::keyboard::KeyCode::*;

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
    fn handle_mouse_move(&mut self, delta: (f32, f32), player_position: &mut Position) -> bool {
        if !self.enabled {
            return false;
        }
        if delta.0.is_zero() && delta.1.is_zero() {
            return false;
        }

        player_position.yaw += Rad(self.turn_speed * delta.0);
        player_position.yaw = player_position.yaw.normalize();

        player_position.pitch -= Rad(self.turn_speed * delta.1);
        // Clip just under fully vertical to avoid weirdness
        player_position.pitch.0 = player_position
            .pitch
            .0
            .clamp(-FRAC_PI_2 * 0.99, FRAC_PI_2 * 0.99);

        true
    }

    /// Update the camera position
    fn move_player(&mut self, player: &mut Player, world: &World, duration: &Duration) -> bool {
        if !self.enabled {
            return false;
        }

        // Step 1: figure out the direction vector the player wants to move in
        let forward = angles_to_vec3(player.position.yaw, player.position.pitch);
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
            return false;
        }
        movement_vector = movement_vector.normalize();

        // Step 2: Figure out if we're colliding with any blocks
        movement_vector *= self.move_speed * duration.as_secs_f32();
        let (movement_vector, _) = predict_collisions(player, world, movement_vector);

        // Apply the movement vector
        player.position.pos.0 += movement_vector;

        true
    }
}
