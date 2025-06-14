use egui::Context;
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{
    CommandEncoder, Device, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};

use crate::render::context::DrawContext;

pub struct UI {
    egui_state: State,
    egui_context: Context,
    egui_renderer: Renderer,
}

impl UI {
    pub fn new(device: &Device, window: &winit::window::Window) -> Self {
        let egui_renderer =
            egui_wgpu::Renderer::new(device, TextureFormat::Bgra8UnormSrgb, None, 1, false);
        let egui_context = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_context.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );

        Self {
            egui_state,
            egui_context,
            egui_renderer,
        }
    }

    /// Render the UI
    pub fn render(
        &mut self,
        draw_context: &DrawContext,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        debug_lines: &[String],
    ) {
        let inputs = egui::RawInput::default();
        let output = self.egui_context.run(inputs, |ctx| {
            // UI code here
            egui::Window::new("Inventory").show(ctx, |ui| {
                debug_lines.iter().for_each(|t| {
                    ui.label(t);
                });
            });
        });

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [draw_context.config.width, draw_context.config.height],
            pixels_per_point: draw_context.window.scale_factor() as f32,
        };

        // Prepare triangles
        let primitives = self
            .egui_context
            .tessellate(output.shapes, screen_descriptor.pixels_per_point);

        // Send new changed textures to GPU
        output
            .textures_delta
            .set
            .iter()
            .for_each(|(id, image_delta)| {
                self.egui_renderer.update_texture(
                    &draw_context.device,
                    &draw_context.queue,
                    *id,
                    image_delta,
                );
            });

        self.egui_renderer.update_buffers(
            &draw_context.device,
            &draw_context.queue,
            encoder,
            &primitives,
            &screen_descriptor,
        );

        {
            // Create a render pass for the UI
            let mut render_pass = encoder
                .begin_render_pass(&RenderPassDescriptor {
                    label: Some("UI Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                })
                .forget_lifetime();

            // Draw the UI
            self.egui_renderer
                .render(&mut render_pass, &primitives, &screen_descriptor);
        }

        // Clean up and un-needed textures
        output.textures_delta.free.iter().for_each(|id| {
            self.egui_renderer.free_texture(id);
        });
    }
}
