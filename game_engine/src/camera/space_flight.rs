use std::{f32::consts::FRAC_PI_2, time::Duration};

use cgmath::{Angle, EuclideanSpace, InnerSpace, Rad, Vector3, Zero};
use hecs::Entity;
use winit::{event::KeyEvent, keyboard::PhysicalKey};

use super::traits::PlayerController;
use crate::{
    camera::collision::predict_collisions,
    entity::components::UprightOrientation,
    math::bbox::AABB,
    state::world::{World, WorldPos},
};

pub struct SpaceFlightController {
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

impl SpaceFlightController {
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

impl PlayerController for SpaceFlightController {
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
    fn handle_mouse_move(
        &mut self,
        ecs: &mut hecs::World,
        entity: Entity,
        delta: (f32, f32),
    ) -> bool {
        if !self.enabled {
            return false;
        }
        if delta.0.is_zero() && delta.1.is_zero() {
            return false;
        }

        let mut orientation = ecs.get::<&mut UprightOrientation>(entity).unwrap();

        orientation.yaw += Rad(self.turn_speed * delta.0);
        orientation.yaw = orientation.yaw.normalize();

        orientation.pitch -= Rad(self.turn_speed * delta.1);
        // Clip just under fully vertical to avoid weirdness
        orientation.pitch.0 = orientation
            .pitch
            .0
            .clamp(-FRAC_PI_2 * 0.99, FRAC_PI_2 * 0.99);

        true
    }

    fn move_entity(
        &mut self,
        ecs: &mut hecs::World,
        entity: Entity,
        world: &World,
        duration: &Duration,
    ) -> bool {
        let mut query = ecs
            .query_one::<(&mut WorldPos, &mut UprightOrientation, &AABB<f32>)>(entity)
            .unwrap();
        let (position, orientation, player_aabb) = query.get().unwrap();
        let player_aabb = player_aabb.translate(&position.0.to_vec());

        // TODO: momentum while we have inventory open?
        if !self.enabled {
            return false;
        }

        // Step 1: figure out the direction vector the player wants to move in
        let forward = orientation.forward();
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
            return false;
        }
        if self.velocity.magnitude2() == 0. {
            return false;
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
        let movement_vector = self.velocity * duration.as_secs_f32();
        let (movement_vector, collisions) =
            predict_collisions(&player_aabb, world, movement_vector);

        // Null velocity in the direction of a collision
        [0, 1, 2].into_iter().for_each(|axis| {
            if collisions[axis].is_some() {
                self.velocity[axis] = 0.;
            }
        });

        if movement_vector.magnitude2() > 0. {
            // Apply the movement vector
            position.0 += movement_vector;

            true
        } else {
            false
        }
    }
}
