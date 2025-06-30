use std::f32::consts::FRAC_PI_2;

use cgmath::{Angle, InnerSpace, Rad, Vector3, Zero};
use winit::{event::KeyEvent, keyboard::PhysicalKey};

use super::{Camera, angles_to_vec3, traits::CameraController};
use crate::{data::block::BlockType, world::BlockPos, world_gen::ChunkGenerator};

pub struct SpaceFlightCameraController {
    acceleration: f32,
    max_speed: Option<f32>,
    drag: f32,
    turn_speed: f32,
    deadstop_speed: f32,

    // Stateful variables
    left_pressed: bool,
    right_pressed: bool,
    up_pressed: bool,
    down_pressed: bool,
    forward_pressed: bool,
    backwards_pressed: bool,
    pub enabled: bool,
    velocity: Vector3<f32>,
}

impl SpaceFlightCameraController {
    pub fn new(acceleration: f32, turn_speed: f32, max_speed: Option<f32>, drag: f32) -> Self {
        assert!(acceleration >= 0.);
        assert!(turn_speed >= 0.);
        assert!(max_speed.is_none_or(|s| s >= 0.));
        assert!(drag >= 0.);
        Self {
            acceleration,
            max_speed,
            drag,
            turn_speed,
            deadstop_speed: 0.5,
            left_pressed: false,
            right_pressed: false,
            up_pressed: false,
            down_pressed: false,
            forward_pressed: false,
            backwards_pressed: false,
            enabled: true,
            velocity: Vector3::zero(),
        }
    }
}

impl CameraController for SpaceFlightCameraController {
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

    fn handle_keypress(&mut self, event: &winit::event::KeyEvent) {
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

    fn update_camera<G: ChunkGenerator>(
        &mut self,
        camera: &mut super::Camera,
        world: &crate::world::World<G>,
        duration: &std::time::Duration,
    ) {
        if !self.enabled {
            return;
        }

        // Step 1: figure out the direction vector the player wants to move in
        let forward = angles_to_vec3(camera.yaw.get(), camera.pitch.get());
        let right = forward.cross(Vector3::unit_y()).normalize();

        let mut acceleration_vector = Vector3::new(0., 0., 0.);
        match (self.left_pressed, self.right_pressed) {
            (true, false) => {
                acceleration_vector -= right;
            }
            (false, true) => {
                acceleration_vector += right;
            }
            _ => {}
        }
        match (self.forward_pressed, self.backwards_pressed) {
            (true, false) => {
                acceleration_vector += forward;
            }
            (false, true) => {
                acceleration_vector -= forward;
            }
            _ => {}
        }
        match (self.up_pressed, self.down_pressed) {
            (true, false) => {
                acceleration_vector.y += 1.;
            }
            (false, true) => {
                acceleration_vector.y -= 1.;
            }
            _ => {}
        }

        // Step 2: Update velocity vector
        if acceleration_vector.magnitude2() > 0. {
            acceleration_vector = acceleration_vector.normalize();
            self.velocity += acceleration_vector * self.acceleration * duration.as_secs_f32();
        }
        if acceleration_vector.magnitude2() == 0.
            && self.velocity.magnitude2() < self.deadstop_speed.powi(2)
        {
            // If we're drifting slowly (0.1 m/s), just stop instead of moving infinitely slowly
            self.velocity.set_zero();
            return;
        }
        if self.velocity.magnitude2() == 0. {
            return;
        }

        // Step 3: Apply drag
        let drag_force = self.velocity.magnitude2() * self.drag * duration.as_secs_f32();
        self.velocity -= self.velocity.normalize_to(drag_force);

        // Step 6: Apply max speed
        if let Some(max_speed) = self.max_speed
            && self.velocity.magnitude2() > 0.
            && self.velocity.magnitude2() > max_speed * max_speed
        {
            self.velocity = self.velocity.normalize_to(max_speed);
        }

        // Step 4: Figure out if we're colliding with any blocks
        let player_block_pos = camera.pos.get().to_block_pos();
        let player_aabb = camera.aabb();

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

        if self.velocity.x != 0.
            && colliding_with(&BlockPos(
                player_block_pos.0 + self.velocity.x.signum() as i32 * Vector3::unit_x(),
            ))
        {
            self.velocity.x = 0.;
        }
        if self.velocity.y != 0.
            && colliding_with(&BlockPos(
                player_block_pos.0 + self.velocity.y.signum() as i32 * Vector3::unit_y(),
            ))
        {
            self.velocity.y = 0.;
        }
        if self.velocity.z != 0.
            && colliding_with(&BlockPos(
                player_block_pos.0 + self.velocity.z.signum() as i32 * Vector3::unit_z(),
            ))
        {
            self.velocity.z = 0.;
        }

        // Step 5: Apply the movement
        camera
            .pos
            .update(|p| p.0 += self.velocity * duration.as_secs_f32());
    }
}
