use std::{f32::consts::FRAC_PI_2, time::Duration};

use cgmath::{Angle, InnerSpace, Rad, Vector3};
use winit::{event::KeyEvent, keyboard::PhysicalKey};

use super::{angles_to_vec3, traits::CameraController};
use crate::{
    camera::Camera,
    world::{BlockPos, World},
    world_gen::ChunkGenerator,
};

/// Handles user input to adjust camera
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
            repeat,
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
    fn update_camera<G: ChunkGenerator>(
        &mut self,
        camera: &mut Camera,
        world: &World<G>,
        duration: &Duration,
    ) {
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
            movement_vector = movement_vector.normalize_to(self.move_speed);
        }

        // Step 2: Figure out if we're colliding with any blocks
        let player_block_pos = camera.pos.get().to_block_pos();
        let player_aabb = camera.aabb();
        let colliding_with = |pos: &BlockPos| {
            if let Some(block) = world.get_block(pos) {
                if block.block_type.is_none() {
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
        if movement_vector.z != 0.
            && colliding_with(&BlockPos(
                player_block_pos.0 + movement_vector.z.signum() as i32 * Vector3::unit_z(),
            ))
        {
            movement_vector.z = 0.;
        }

        // Step 3: Verical
        if self.vertical_velocity > 0.
            && colliding_with(&BlockPos(player_block_pos.0 + Vector3::unit_y()))
        {
            // Hit our head on the roof
            self.vertical_velocity = 0.;
        }

        let on_floor = colliding_with(&BlockPos(player_block_pos.0 - Vector3::unit_y()));
        if !on_floor {
            // Apply gravity
            self.vertical_velocity -= self.gravity * duration.as_secs_f32();
        } else if self.vertical_velocity < 0. {
            // Land on the floor
            self.vertical_velocity = 0.;
        }

        // Step 3: Jumping
        if self.up_pressed && on_floor && self.vertical_velocity <= 0. {
            self.vertical_velocity += self.jump_force;
        }

        movement_vector.y = self.vertical_velocity;

        // TODO: Need to do correct collision detection at high speeds so we don't get stuck inside
        // walls

        // Apply the movement vector
        camera
            .pos
            .update(|p| p.0 += movement_vector * duration.as_secs_f32());
    }
}
