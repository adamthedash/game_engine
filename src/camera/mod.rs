pub mod basic_flight;
pub mod space_flight;
pub mod traits;
pub mod walking;

use cgmath::{
    Angle, Deg, InnerSpace, Matrix4, Rad, SquareMatrix, Transform, Vector3, Vector4, perspective,
};

use crate::{bbox::AABB, render::ray::Ray, world::WorldPos};

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

    /// Checks whether a point is within the viewport
    pub fn in_view(&self, pos: &WorldPos) -> bool {
        let view_proj_matrix = self.get_view_proj_matrix();
        let projected = view_proj_matrix.transform_point(pos.0);

        // Check if projected point is within the -1 .. 1 NCD
        projected.x.abs() <= 1. || projected.y.abs() <= 1. || projected.z.abs() <= 1.
    }

    /// Return the bounding box of the camera
    pub fn aabb(&self) -> AABB<f32> {
        let height = 1.8;
        let width = 0.8;
        let head_height = 1.8;

        let diff = Vector3::new(width / 2., height / 2., width / 2.);
        let head_diff = Vector3::unit_y() * head_height / 2.;

        AABB::new(
            &(self.pos.0 - diff - head_diff),
            &(self.pos.0 + diff - head_diff),
        )
    }

    pub fn ray(&self) -> Ray {
        Ray::new(self.pos.0, angles_to_vec3(self.yaw, self.pitch))
    }
}

/// Projection matrix for the shaders, stored on the GPU
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
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

fn angles_to_vec3(yaw: Rad<f32>, pitch: Rad<f32>) -> Vector3<f32> {
    let (y, verticality) = pitch.sin_cos();
    let (z, x) = yaw.sin_cos();
    Vector3::new(x * verticality, y, z * verticality).normalize()
}
