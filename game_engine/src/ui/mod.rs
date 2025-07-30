pub mod axes;
pub mod crafting;
pub mod debug;
pub mod hotbar;
pub mod inventory;

use axes::Axes;
use egui::{Color32, Context, FontId, ImageSource, Response, Ui, Vec2};
use egui_taffy::{TuiBuilderLogic, TuiWidget};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{
    CommandEncoder, Device, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, TextureFormat, TextureView,
};

use crate::{
    InteractionMode,
    render::{camera::Camera, context::DrawContext},
    state::game::GameState,
    ui::{crafting::CraftingWindow, debug::DEBUG_WINDOW},
};

/// Trait to enable easy drawing of UI elements
pub trait Drawable {
    /// Draw a new window with the UI
    fn show_window(&self, _ctx: &Context) {}
    /// Draw the UI in an existing window
    fn show_widget(&self, _ui: &mut Ui) {}
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
        camera: &Camera,
        view: &TextureView,
        game: &GameState,
        game_mode: &InteractionMode,
    ) {
        let inputs = self.egui_state.take_egui_input(&draw_context.window);
        let output = self.egui_context.run(inputs, |ctx| {
            // UI code here
            DEBUG_WINDOW.show_window(ctx);

            Axes { camera }.show_window(ctx);

            match game_mode {
                InteractionMode::Game => {}
                InteractionMode::UI => {
                    game.player.inventory.borrow().show_window(ctx);

                    CraftingWindow {
                        inventory: game.player.inventory.clone(),
                    }
                    .show_window(ctx);
                }
                InteractionMode::Block(block_pos) => {
                    let block_state = game
                        .world
                        .get_block_state(block_pos)
                        .unwrap_or_else(|| panic!("Block state doesn't exist for {block_pos:?}"));

                    // TODO: Only show inventory when block has some interaction with it?
                    game.player.inventory.borrow().show_window(ctx);
                    block_state.show_window(ctx);
                }
            }
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

#[derive(Clone)]
pub struct Icon<'a> {
    pub texture: &'a ImageSource<'static>,
    pub size: f32,
    pub count: Option<usize>,
    pub font_size: f32,
}

impl<'a> Icon<'a> {
    pub fn draw(&self, ui: &mut Ui) -> Response {
        let resp = ui
            .add(egui::Image::new(self.texture.clone()).fit_to_exact_size(Vec2::splat(self.size)));
        let rect = resp.rect;

        // Draw item count in bottom right
        if let Some(count) = self.count {
            let painter = ui.painter();
            let font_id = FontId::monospace(self.font_size);
            let text = painter.layout_no_wrap(format!("{count}"), font_id, Color32::WHITE);

            let pos = rect.right_bottom() - text.size();
            painter.galley(pos, text, Color32::WHITE);
        }

        resp
    }
}

impl<'a> TuiWidget for Icon<'a> {
    type Response = egui::Response;

    fn taffy_ui(self, tuib: egui_taffy::TuiBuilder) -> Self::Response {
        tuib.ui_add_manual(|ui| self.draw(ui), |resp, _| resp)
    }
}
