use std::{f32::consts::FRAC_PI_2, time::Duration};

use cgmath::{Angle, InnerSpace, Rad, Vector3};
use winit::{event::KeyEvent, keyboard::PhysicalKey};

use super::{angles_to_vec3, traits::CameraController};
use crate::{
    camera::{Camera, collision::predict_collisions},
    state::world::World,
};

/// Handles user input to adjust camera
#[derive(Debug)]
pub struct WalkingCameraController {
    move_speed: f32,
    turn_speed: f32,
    gravity: f32,
    jump_force: f32,
    // Stateful variables
    left_pressed: bool,
    right_pressed: bool,
    up_pressed: bool,
    forward_pressed: bool,
    backwards_pressed: bool,
    pub enabled: bool,
    vertical_velocity: f32,
}

impl WalkingCameraController {
    pub fn new(move_speed: f32, turn_speed: f32, gravity: f32, jump_height: f32) -> Self {
        assert!(move_speed >= 0.);
        assert!(turn_speed >= 0.);
        assert!(gravity >= 0.);
        assert!(jump_height >= 0.);

        let jump_force = (2. * gravity * jump_height).sqrt();
        Self {
            move_speed,
            turn_speed,
            gravity,
            jump_force,
            left_pressed: false,
            right_pressed: false,
            up_pressed: false,
            forward_pressed: false,
            backwards_pressed: false,
            enabled: true,
            vertical_velocity: 0.,
        }
    }
}

impl CameraController for WalkingCameraController {
    fn toggle(&mut self) {
        if self.enabled {
            // Un-press any buttons
            self.left_pressed = false;
            self.right_pressed = false;
            self.up_pressed = false;
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
                _ => {}
            }
        }
    }

    /// Turn the camera. delta is in normalised screen coordinates -1 to 1
    fn handle_mouse_move(&mut self, delta: (f32, f32), camera: &mut Camera) {
        if !self.enabled {
            return;
        }

        camera.yaw += Rad(self.turn_speed * delta.0);
        camera.yaw.set(camera.yaw.get().normalize());

        camera.pitch -= Rad(self.turn_speed * delta.1);
        // Clip just under fully vertical to avoid weirdness
        camera
            .pitch
            .update(|p| p.0 = p.0.clamp(-FRAC_PI_2 * 0.99, FRAC_PI_2 * 0.99));
    }

    /// Update the camera position
    fn update_camera(&mut self, camera: &mut Camera, world: &World, duration: &Duration) {
        // Step 1: figure out the direction vector the player wants to move in
        let forward = angles_to_vec3(camera.yaw.get(), Rad(0.));
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
        if movement_vector.magnitude2() > 0. {
            movement_vector =
                movement_vector.normalize_to(self.move_speed * duration.as_secs_f32());
        }

        // Gravity
        self.vertical_velocity -= self.gravity * duration.as_secs_f32();
        movement_vector.y = self.vertical_velocity * duration.as_secs_f32();

        // Figure out if we're colliding with any blocks
        // TODO: Fix momentum based controllers after collision detection is fixed
        let (movement_vector, collisions) = predict_collisions(camera, world, movement_vector);
        if collisions[1].is_some() {
            self.vertical_velocity = 0.;
        }

        // Jumping
        if self.up_pressed && collisions[1].is_some_and(|y| y <= 0.) {
            self.vertical_velocity = self.jump_force;
        }

        // Apply the movement vector
        camera.pos.update(|p| p.0 += movement_vector);
    }
}
