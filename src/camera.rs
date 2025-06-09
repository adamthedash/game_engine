use cgmath::{Deg, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, Vector4, perspective};
use winit::{event::ElementState, keyboard::KeyCode};

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
    pub pos: Point3<f32>,
    pub forward: Vector3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn get_view_proj_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.pos, self.pos + self.forward, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
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
        }
    }

    /// Update the movement state based on user key presses
    pub fn handle_keypress(&mut self, key: KeyCode, state: ElementState) {
        use KeyCode::*;
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

    pub fn handle_mouse_move(&mut self, axis: u32, value: f64) {
        match axis {
            0 => {}
            1 => {}
            _ => {}
        }
    }

    /// Update the camera position
    pub fn update_camera(&self, camera: &mut Camera) {
        match (self.left_pressed, self.right_pressed) {
            (true, false) => {
                let right = camera.forward.cross(camera.up);
                camera.pos -= right * self.move_speed;
            }
            (false, true) => {
                let right = camera.forward.cross(camera.up);
                camera.pos += right * self.move_speed;
            }
            _ => {}
        }
        match (self.forward_pressed, self.backwards_pressed) {
            (true, false) => {
                camera.pos += camera.forward * self.move_speed;
            }
            (false, true) => {
                camera.pos -= camera.forward * self.move_speed;
            }
            _ => {}
        }
        match (self.up_pressed, self.down_pressed) {
            (true, false) => {
                camera.pos += camera.up.normalize() * self.move_speed;
            }
            (false, true) => {
                camera.pos -= camera.up.normalize() * self.move_speed;
            }
            _ => {}
        }
    }
}
