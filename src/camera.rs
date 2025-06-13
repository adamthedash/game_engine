use std::{f32::consts::FRAC_PI_2, time::Duration};

use cgmath::{
    Angle, Deg, ElementWise, InnerSpace, Matrix4, Rad, SquareMatrix, Vector3, Vector4, perspective,
};
use winit::{event::KeyEvent, keyboard::PhysicalKey};

use crate::{
    bbox::AABB,
    world::{BlockPos, BlockType, World, WorldPos},
};

/// Matrix used to convert from OpenGL to WebGPU NCD
const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::from_cols(
    Vector4::new(1.0, 0.0, 0.0, 0.0),
    Vector4::new(0.0, 1.0, 0.0, 0.0),
    Vector4::new(0.0, 0.0, 0.5, 0.0),
    Vector4::new(0.0, 0.0, 0.5, 1.0),
);

/// Holds the current state of the camera
#[derive(Debug)]
pub struct Camera {
    pub pos: WorldPos,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub aspect: f32,
    pub fovy: Deg<f32>,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn get_view_proj_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        let view = Matrix4::look_to_rh(
            self.pos.0,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        );
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

/// Projection matrix for the shaders, stored on the GPU
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }

    /// This should always be called whenever the camera is updated
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.get_view_proj_matrix().into();
    }
}

/// Handles user input to adjust camera
pub struct CameraController {
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

impl CameraController {
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

    /// Update the movement state based on user key presses
    pub fn handle_keypress(&mut self, event: &KeyEvent) {
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
    pub fn handle_mouse_move(&mut self, axis: u32, delta: f32, camera: &mut Camera) {
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
    pub fn update_camera(&self, camera: &mut Camera, world: &World, duration: &Duration) {
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

fn angles_to_vec3(yaw: Rad<f32>, pitch: Rad<f32>) -> Vector3<f32> {
    let (y, verticality) = pitch.sin_cos();
    let (z, x) = yaw.sin_cos();
    Vector3::new(x * verticality, y, z * verticality).normalize()
}
