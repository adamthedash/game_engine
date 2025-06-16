pub mod debug;
pub mod hotbar;
pub mod inventory;

use debug::DebugWindow;
use egui::{Context, Ui};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{
    CommandEncoder, Device, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, TextureFormat, TextureView,
};

use crate::{InteractionMode, game::GameState, render::context::DrawContext};

/// Trait to enable easy drawing of UI elements
pub trait Drawable {
    fn show_window(&self, ctx: &Context);
    fn show_widget(&self, ui: &mut Ui);
}

pub struct UI {
    // Rendering
    pub egui_state: State,
    egui_context: Context,
    pub egui_renderer: Renderer,
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
        }
    }

    /// Render the UI
    pub fn render(
        &mut self,
        draw_context: &DrawContext,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        game: &GameState,
        game_mode: &InteractionMode,
        debug_lines: &[String],
    ) {
        let inputs = self.egui_state.take_egui_input(&draw_context.window);
        let output = self.egui_context.run(inputs, |ctx| {
            // UI code here
            DebugWindow {
                lines: debug_lines.to_vec(),
            }
            .show_window(ctx);

            match game_mode {
                InteractionMode::Game => {
                    game.player.hotbar.show_window(ctx);
                }
                InteractionMode::UI => {
                    game.player.inventory.show_window(ctx);
                }
            }
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

        // Clean up any un-needed textures
        output.textures_delta.free.iter().for_each(|id| {
            self.egui_renderer.free_texture(id);
        });
    }
}
