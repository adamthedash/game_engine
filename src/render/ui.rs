use egui::{Context, Window, include_image};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{
    CommandEncoder, Device, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, TextureFormat, TextureView,
};

use crate::render::context::DrawContext;

pub struct UI {
    // Rendering
    pub egui_state: State,
    egui_context: Context,
    egui_renderer: Renderer,
    // State
    pub hotbar_selected: usize,
    pub num_hotbars: usize,
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

        egui_extras::install_image_loaders(&egui_context);

        Self {
            egui_state,
            egui_context,
            egui_renderer,
            hotbar_selected: 0,
            num_hotbars: 10,
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
        let inputs = self.egui_state.take_egui_input(&draw_context.window);
        let output = self.egui_context.run(inputs, |ctx| {
            // UI code here
            Window::new("Debug").default_open(false).show(ctx, |ui| {
                debug_lines.iter().for_each(|t| {
                    ui.label(t);
                });
            });

            Window::new("Hotbar")
                .title_bar(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.columns(self.num_hotbars, |columns| {
                        columns.iter_mut().enumerate().for_each(|(i, c)| {
                            if i == self.hotbar_selected {
                                c.image(include_image!("../../res/meshes/smiley2.png"));
                            } else {
                                c.image(include_image!("../../res/meshes/smiley.png"));
                            }
                        });
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

    /// Move the selected hotbar up or down by one
    pub fn scroll_hotbar(&mut self, up: bool) {
        if up {
            self.hotbar_selected += 1;
            self.hotbar_selected %= self.num_hotbars;
        } else {
            if self.hotbar_selected == 0 {
                self.hotbar_selected += self.num_hotbars;
            }
            self.hotbar_selected -= 1;
        }
    }
}
