use cgmath::{InnerSpace, Matrix4, Transform, Vector3, perspective};
use egui::{Color32, Sense, Stroke, Vec2, Window};

use super::Drawable;
use crate::camera::{Camera, OPENGL_TO_WGPU_MATRIX};

/// Representation of the world axes
pub struct Axes<'a> {
    pub camera: &'a Camera,
}

impl Drawable for Axes<'_> {
    fn show_window(&self, ctx: &egui::Context) {
        let window_size = 256.;
        let arrow_length = window_size / 2.;

        let axes = [Vector3::unit_x(), Vector3::unit_y(), Vector3::unit_z()];
        let colors = [Color32::RED, Color32::GREEN, Color32::BLUE];

        // Same as camera MVP except for aspect ratio.
        let transform = {
            let (sin_pitch, cos_pitch) = self.camera.pitch.get().0.sin_cos();
            let (sin_yaw, cos_yaw) = self.camera.yaw.get().0.sin_cos();

            let view = Matrix4::look_to_rh(
                self.camera.pos.get().0,
                Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
                Vector3::unit_y(),
            );
            let proj = perspective(
                self.camera.fovy.get(),
                1., // Fixed aspect ratio otherwise the axes get all skewed
                self.camera.znear.get(),
                self.camera.zfar.get(),
            );

            OPENGL_TO_WGPU_MATRIX * proj * view
        };

        Window::new("Axes").default_open(false).show(ctx, |ui| {
            let (resp, painter) = ui.allocate_painter(Vec2::splat(window_size), Sense::empty());

            // Find origin point of window in camera space
            let centre = resp.rect.center();
            let screen_size = ctx.screen_rect().size();
            // -1 .. 1 screen space. (-1, -1) is top-left
            let normalised_centre = Vec2::new(centre.x / screen_size.x, centre.y / screen_size.y)
                * 2.
                - Vec2::splat(1.);

            // Get vector out from camera through UI centre
            let world_vector = self
                .camera
                .viewport_to_world_vector(normalised_centre.to_pos2());

            // Position of window centre in world pos
            let axis_origin_world = self.camera.pos.get().0 + world_vector * 3.;

            // Transform axes relative to this centre point
            let axis_origin_transformed = transform.transform_point(axis_origin_world);

            axes.into_iter()
                // Transform the unit vectors into camera space
                .map(|axis| {
                    transform.transform_point(axis_origin_world + axis) - axis_origin_transformed
                })
                .zip(colors)
                // Draw them on the screen
                .for_each(|(vector, color)| {
                    // Truncate 3D -> 2D
                    // Gui coords are top-down, so flip Y
                    let vec = Vec2::new(vector.x, -vector.y) * arrow_length;

                    // Draw the arrow from the centre of the window
                    painter.arrow(resp.rect.center(), vec, Stroke::new(1., color));
                });
        });
    }
}
