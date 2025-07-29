use cgmath::{
    Angle, Deg, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Transform, Vector3, Vector4,
    perspective,
};
use egui::Pos2;
use sycamore_reactive::{ReadSignal, Signal, create_memo, create_signal};

use crate::{
    math::{
        angles_to_vec3,
        bbox::AABB,
        frustum::{Frustum, Plane},
    },
    state::world::WorldPos,
};

/// Matrix used to convert from OpenGL to WebGPU NCD
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::from_cols(
    Vector4::new(1.0, 0.0, 0.0, 0.0),
    Vector4::new(0.0, 1.0, 0.0, 0.0),
    Vector4::new(0.0, 0.0, 0.5, 0.0),
    Vector4::new(0.0, 0.0, 0.5, 1.0),
);

/// Holds the current state of the camera
#[derive(Debug)]
pub struct Camera {
    pub pos: Signal<WorldPos>,
    pub yaw: Signal<Rad<f32>>,
    pub pitch: Signal<Rad<f32>>,
    pub aspect: Signal<f32>,
    pub fovy: Signal<Deg<f32>>,
    pub znear: Signal<f32>,
    pub zfar: Signal<f32>,
    pub view_proj_matrix: ReadSignal<Matrix4<f32>>,
    pub frustum: ReadSignal<Frustum>,
}

/// Represents coordinates in -1 .. 1 NCD space
#[derive(Debug)]
pub struct NCDPos(pub Point3<f32>);

impl Camera {
    pub fn new(
        pos: WorldPos,
        yaw: Rad<f32>,
        pitch: Rad<f32>,
        aspect: f32,
        fovy: Deg<f32>,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let pos = create_signal(pos);
        let yaw = create_signal(yaw);
        let pitch = create_signal(pitch);
        let aspect = create_signal(aspect);
        let fovy = create_signal(fovy);
        let znear = create_signal(znear);
        let zfar = create_signal(zfar);

        let view_proj_matrix = create_memo(move || {
            let (sin_pitch, cos_pitch) = pitch.get().0.sin_cos();
            let (sin_yaw, cos_yaw) = yaw.get().0.sin_cos();

            let view = Matrix4::look_to_rh(
                pos.get().0,
                Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
                Vector3::unit_y(),
            );
            let proj = perspective(fovy.get(), aspect.get(), znear.get(), zfar.get());

            OPENGL_TO_WGPU_MATRIX * proj * view
        });

        let frustum = create_memo(move || {
            let forward = angles_to_vec3(yaw.get(), pitch.get());
            let right = forward.cross(Vector3::unit_y()).normalize();
            let up = right.cross(forward).normalize();

            // Convert vertical FOV from degrees to radians
            let half_v_fov = fovy.get() * 0.5;
            let half_h_fov = Deg::from(Rad((half_v_fov.tan() * aspect.get()).atan()));

            // Near and far plane centers
            let near_center = pos.get().0 + forward * znear.get();
            let far_center = pos.get().0 + forward * zfar.get();

            // Create planes with inward-pointing normals
            let near = Plane::from_normal_point(&forward, &near_center);
            let far = Plane::from_normal_point(&(-forward), &far_center);

            let left_normal = (right * half_h_fov.cos() + forward * half_h_fov.sin()).normalize();
            let left = Plane::from_normal_point(&left_normal, &pos.get().0);
            let right_normal = (-right * half_h_fov.cos() + forward * half_h_fov.sin()).normalize();
            let right = Plane::from_normal_point(&right_normal, &pos.get().0);

            let top_normal = (-up * half_v_fov.cos() + forward * half_v_fov.sin()).normalize();
            let top = Plane::from_normal_point(&top_normal, &pos.get().0);
            let bottom_normal = (up * half_v_fov.cos() + forward * half_v_fov.sin()).normalize();
            let bottom = Plane::from_normal_point(&bottom_normal, &pos.get().0);

            Frustum {
                near,
                far,
                top,
                bottom,
                left,
                right,
            }
        });

        Self {
            pos,
            yaw,
            pitch,
            aspect,
            fovy,
            znear,
            zfar,
            view_proj_matrix,
            frustum,
        }
    }

    /// Project a point into -1 .. 1 NCD coordinates
    pub fn project_to_ncd(&self, pos: &WorldPos) -> NCDPos {
        NCDPos(self.view_proj_matrix.get().transform_point(pos.0))
    }

    /// Checks whether a point is within the viewport
    pub fn in_view_point(&self, pos: &WorldPos) -> bool {
        self.frustum.with(|f| f.contains_point(pos))
    }

    /// Checks whether an AABB has any overlap with the viewport
    pub fn in_view_aabb(&self, aabb: &AABB<f32>) -> bool {
        self.frustum.with(|f| f.intersects_aabb(aabb))
    }

    /// Convert a point on screen space (-1 .. 1, top-left origin) to a vector in that direction in world space.
    pub fn viewport_to_world_vector(&self, viewport_pos: Pos2) -> Vector3<f32> {
        // Get axis vector in camera space
        let forward = angles_to_vec3(self.yaw.get(), self.pitch.get());
        let right = forward.cross(Vector3::unit_y()).normalize();
        let up = right.cross(forward).normalize();

        // Size of plane in world units
        let near_plane_height = (self.fovy.get() / 2.).tan() * self.znear.get() * 2.;
        let near_plane_width = near_plane_height * self.aspect.get();

        // Vector from camera origin through viewport point in world space.
        let centre_world_vector = forward * self.znear.get()
            + right * near_plane_width * viewport_pos.x / 2.
            + up * near_plane_height * (-viewport_pos.y) / 2.;

        centre_world_vector.normalize()
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
        self.view_proj = camera.view_proj_matrix.get().into();
    }
}
