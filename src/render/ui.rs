use egui::{Align2, Color32, Context, Frame, Stroke, Ui, Window};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{
    CommandEncoder, Device, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, TextureFormat, TextureView,
};

use crate::{game::GameState, inventory::Hotbar, item::ITEMS, render::context::DrawContext};

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

            game.player.hotbar.show_window(ctx);
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

/// Trait to enable easy drawing of UI elements
pub trait Drawable {
    fn show_window(&self, ctx: &Context);
    fn show_widget(&self, ui: &mut Ui);
}

impl Drawable for Hotbar {
    fn show_window(&self, ctx: &Context) {
        Window::new("Hotbar")
            .title_bar(false)
            .resizable(false)
            .frame(Frame::window(&ctx.style()).inner_margin(0))
            .anchor(Align2::CENTER_BOTTOM, [0., 0.])
            .show(ctx, |ui| {
                // Remove horizontal padding ebtween slots
                ui.spacing_mut().item_spacing.x = 0.;

                self.show_widget(ui);
            });
    }

    fn show_widget(&self, ui: &mut Ui) {
        let icon_size = 32.;
        let selected_margin_size = 3.;

        let items = ITEMS.get().expect("Item info has not been initialised!");

        ui.columns(self.slots.len(), |columns| {
            // Draw item slots
            columns.iter_mut().enumerate().for_each(|(i, c)| {
                let frame = Frame::new().stroke(Stroke::new(
                    selected_margin_size,
                    // Highlight selected slot
                    if i == self.selected {
                        Color32::LIGHT_BLUE
                    } else {
                        Color32::TRANSPARENT
                    },
                ));

                // Get icon for the item
                let icon = self.slots[i]
                    .and_then(|id| items.get(&id).and_then(|item| item.icon.as_ref()))
                    .unwrap_or_else(|| items.get(&0).unwrap().icon.as_ref().unwrap());

                frame.show(c, |ui| {
                    ui.add(
                        egui::Image::new(icon.clone())
                            .fit_to_exact_size([icon_size, icon_size].into()),
                    );
                });
            });
        });
    }
}
