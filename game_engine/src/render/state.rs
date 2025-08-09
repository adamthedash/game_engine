use std::{path::Path, sync::Arc};

use anyhow::Context;
use cgmath::{EuclideanSpace, Matrix3, Matrix4, One};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferDescriptor,
    BufferUsages, CommandEncoderDescriptor, Device, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::{dpi::PhysicalSize, window::Window};

use super::shaders::texture;
use crate::{
    InteractionMode,
    block::Block,
    data::loader::{BLOCK_TEXTURES, BLOCKS, init_block_info, init_item_info},
    entity::components::{self, Vision},
    event::{Message, Subscriber},
    render::{
        camera::{Camera, CameraUniform},
        context::DrawContext,
        light::LightingUniform,
        model::{Mesh, Model},
        shaders::{
            lighting::{LightingShaderPipeline, LightingShaderPipelineLayout},
            texture::{TextureShaderPipeline, TextureShaderPipelineLayout},
            wireframe::{self, WireframeShaderPipeline, WireframeShaderPipelineLayout},
        },
        texture::Texture,
    },
    state::{
        game::GameState,
        world::{Chunk, WorldPos},
    },
    ui::{UI, debug::DEBUG_WINDOW},
    util::{counter::Counter, stopwatch::StopWatch},
};

/// Create a new instance buffer for the given type
pub fn create_instance_buffer<T>(device: &Device, max_elements: usize, name: &str) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some(&format!("Instance Buffer: {name:?}")),
        size: (std::mem::size_of::<T>() * max_elements) as u64,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// Holds all of the stuff related to rendering the game window.
pub struct RenderState {
    // GPU and window stuff
    pub draw_context: DrawContext,
    // Shader stuff
    texture_shader_pipeline: TextureShaderPipeline,
    wireframe_pipeline: WireframeShaderPipeline,
    lighting_shader_pipeline: LightingShaderPipeline,
    depth_texture: Texture,
    block_texture_bind_group: BindGroup,
    // Instance buffers
    block_textured_instance_buffer: Buffer,
    block_wireframe_instance_buffer: Buffer,
    sibeal_instance_buffer: Buffer,
    // Entity stuff
    block_model: Model,
    sibeal_model: Model,
    block_wireframe_mesh: Mesh,
    // Camera stuff
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera: Camera,
    // Lighting stuff
    _lighting_uniform: LightingUniform,
    _lighting_buffer: Buffer,
    // GUI stuff
    pub ui: UI,
    // Re-usable CPU buffers
    instances_cpu: Vec<texture::Instance>,
    visible_blocks: Vec<Block>,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let draw_context = DrawContext::new(window).await;

        // Shaders
        let texture_shader = TextureShaderPipelineLayout::new(&draw_context.device);
        let light_shader = LightingShaderPipelineLayout::new(&draw_context.device);
        let wireframe_shader = WireframeShaderPipelineLayout::new(&draw_context.device);

        // Texture
        let depth_texture = Texture::create_depth_texture(
            &draw_context.device,
            &draw_context.config,
            "Depth Texture",
        );

        // Mesh
        let block_path = Path::new("res/meshes/block.obj");
        let block_model = Model::load_model(
            block_path,
            &draw_context.device,
            &draw_context.queue,
            &texture_shader.texture_layout,
        )
        .with_context(|| format!("Failed to load block model: {block_path:?}"))
        .unwrap();
        let sibeal_path = Path::new("res/meshes/sibeal.obj");
        let sibeal_model = Model::load_model(
            sibeal_path,
            &draw_context.device,
            &draw_context.queue,
            &texture_shader.texture_layout,
        )
        .with_context(|| format!("Failed to load sibeal model: {block_path:?}"))
        .unwrap();

        // Camera
        let (camera_uniform, camera_buffer) = RenderState::init_camera(&draw_context.device);
        let (lighting_uniform, lighting_buffer) = RenderState::init_lighting(&draw_context.device);

        // Render Pipeline
        let texture_shader_pipeline = texture_shader.init(
            &draw_context.device,
            &draw_context.config.format,
            &camera_buffer,
            &lighting_buffer,
        );
        let lighting_shader_pipeline = light_shader.init(
            &draw_context.device,
            &draw_context.config.format,
            &camera_buffer,
            &lighting_buffer,
        );
        let wireframe_pipeline = wireframe_shader.init(
            &draw_context.device,
            &draw_context.config.format,
            &camera_buffer,
        );

        // Renderables
        let cube_wireframe_vertices: [[f32; 3]; 8] = [
            [0., 0., 0.],
            [1., 0., 0.],
            [1., 1., 0.],
            [0., 1., 0.],
            [0., 0., 1.],
            [1., 0., 1.],
            [1., 1., 1.],
            [0., 1., 1.],
        ];
        let cube_wireframe_indices: [u32; 24] = [
            // Bottom face edges
            0, 1, // bottom-back-left to bottom-back-right
            1, 2, // bottom-back-right to top-back-right
            2, 3, // top-back-right to top-back-left
            3, 0, // top-back-left to bottom-back-left
            // Top face edges
            4, 5, // bottom-front-left to bottom-front-right
            5, 6, // bottom-front-right to top-front-right
            6, 7, // top-front-right to top-front-left
            7, 4, // top-front-left to bottom-front-left
            // Vertical edges connecting bottom and top faces
            0, 4, // bottom-back-left to bottom-front-left
            1, 5, // bottom-back-right to bottom-front-right
            2, 6, // top-back-right to top-front-right
            3, 7, // top-back-left to top-front-left
        ];
        let block_wireframe_mesh = Mesh::new(
            &draw_context.device,
            &cube_wireframe_vertices,
            &cube_wireframe_indices,
            "Block Wireframe",
        );
        let block_wireframe_instance_buffer = create_instance_buffer::<wireframe::Instance>(
            &draw_context.device,
            1,
            "Block Wireframe",
        );

        // Instances of blocks
        let block_textured_instance_buffer = create_instance_buffer::<texture::Instance>(
            &draw_context.device,
            Chunk::BLOCKS_PER_CHUNK * 512,
            "Block Textured",
        );
        let sibeal_instance_buffer =
            create_instance_buffer::<texture::Instance>(&draw_context.device, 1, "Sibeal");

        // GUI
        let mut ui = UI::new(&draw_context.device, &draw_context.window);

        // Item textures
        init_item_info(&draw_context, &mut ui.egui_renderer);
        init_block_info(&draw_context);

        let block_texture_bind_group =
            draw_context.device.create_bind_group(&BindGroupDescriptor {
                layout: &texture_shader_pipeline.layouts.texture_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&BLOCK_TEXTURES.get().unwrap().view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&BLOCK_TEXTURES.get().unwrap().sampler),
                    },
                ],
                label: Some("Bind group: Block Textures"),
            });

        Self {
            draw_context,
            texture_shader_pipeline,
            block_model,
            sibeal_model,
            camera_uniform,
            camera_buffer,
            block_textured_instance_buffer,
            sibeal_instance_buffer,
            depth_texture,
            _lighting_uniform: lighting_uniform,
            _lighting_buffer: lighting_buffer,
            lighting_shader_pipeline,
            ui,
            instances_cpu: vec![],
            visible_blocks: vec![],
            wireframe_pipeline,
            block_wireframe_instance_buffer,
            block_wireframe_mesh,
            block_texture_bind_group,
            camera: Camera::default(),
        }
    }

    fn init_camera(device: &Device) -> (CameraUniform, Buffer) {
        let uniform = CameraUniform::new();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        (uniform, buffer)
    }

    fn init_lighting(device: &Device) -> (LightingUniform, Buffer) {
        let uniform = LightingUniform::new([1., 9., -16.], [1., 1., 1.]);
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Lighting Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        (uniform, buffer)
    }

    // Resize the window
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.draw_context.resize(size);
        self.depth_texture = Texture::create_depth_texture(
            &self.draw_context.device,
            &self.draw_context.config,
            "depth_texture",
        );

        self.camera
            .aspect
            .set(size.width as f32 / size.height as f32);
        self.update_camera_buffer();
    }

    /// Syncs the camera state to the buffer being rendered in the GPU
    pub fn update_camera_buffer(&mut self) {
        self.camera_uniform.update_view_proj(&self.camera);
        self.draw_context.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    /// Perform the actual rendering to the screen
    pub fn render(&mut self, game: &GameState) {
        let mut stopwatch = StopWatch::new();
        let mut total_stopwatch = StopWatch::new();
        let mut counter = Counter::new();
        counter.enabled = false;

        let mut query = game
            .ecs
            .query_one::<(&WorldPos, &Vision)>(game.player)
            .unwrap();
        let (player_pos, vision_distance) = query.get().unwrap();
        let player_target_block = game.get_player_target_block();

        // Check what blocks are candidates for rendering
        let (player_chunk, _) = player_pos.to_block_pos().to_chunk_offset();
        let player_vision_chunks = (vision_distance.0 as u32).div_ceil(Chunk::CHUNK_SIZE as u32);

        let blocks = BLOCKS.get().expect("Block data not initalised!");
        self.visible_blocks.clear();
        player_chunk
            // Only render chunks within vision distance of the player (plus 1 chunk buffer)
            .chunks_within(player_vision_chunks + 1)
            .inspect(|_| {
                counter.increment("Chunks in range");
            })
            // Only render chunks in the player's viewport
            .filter(|chunk_pos| self.camera.in_view_aabb(&chunk_pos.aabb().to_f32()))
            .inspect(|_| {
                counter.increment("Chunks in view");
            })
            .flat_map(|pos| game.world.chunks.get(&pos))
            .flat_map(|chunk| {
                chunk
                    .iter_blocks()
                    .inspect(|_| {
                        counter.increment("All blocks");
                    })
                    // Don't render air blocks
                    .filter(|b| blocks[b.block_type].data.renderable)
                    .inspect(|_| {
                        counter.increment("Non-air blocks");
                    })
                    // Only render exposed blocks
                    .filter(|b| {
                        let (_, block_pos) = b.block_pos.to_chunk_offset();
                        chunk.is_block_exposed(block_pos)
                    })
                    .inspect(|_| {
                        counter.increment("Exposed blocks");
                    })
            })
            .inspect(|_| {
                counter.increment("Blocks rendered");
            })
            .collect_into(&mut self.visible_blocks);

        stopwatch.stamp_and_reset("Block elimination");

        // Convert blocks to renderable instances
        self.instances_cpu.clear();
        self.visible_blocks
            .iter()
            .map(|block| block.to_instance())
            .collect_into(&mut self.instances_cpu);
        stopwatch.stamp_and_reset("Instance creation");

        self.draw_context.queue.write_buffer(
            &self.block_textured_instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances_cpu),
        );

        // Entities

        let mut sibeal_query = game
            .ecs
            .query::<(&components::Position, &components::Orientation)>();
        let (_, (sibeal_pos, sibeal_rot)) = sibeal_query.iter().next().unwrap();

        let sibeal_instances = [texture::Instance {
            model: (Matrix4::from_translation(sibeal_pos.0.0.to_vec())
                * Matrix4::from(sibeal_rot.0))
            .into(),
            normal: Matrix3::one().into(),
            texture_index: 0,
        }];
        self.draw_context.queue.write_buffer(
            &self.sibeal_instance_buffer,
            0,
            bytemuck::cast_slice(&sibeal_instances),
        );

        // Get a view on the surface texture that we'll draw to
        let (output, texture_view) = self.draw_context.get_texture_view();

        // Encoder is used to send operations to the GPU queue
        let mut encoder =
            self.draw_context
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            // Clear the screen and fill it with blue
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.8,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            // Draw our mesh cubes
            let mesh = &self.block_model.meshes[0];
            self.texture_shader_pipeline.draw(
                &mut render_pass,
                mesh,
                &self.block_texture_bind_group,
                &self.block_textured_instance_buffer,
                self.instances_cpu.len(),
            );

            // Draw Sibeal
            let mesh = &self.sibeal_model.meshes[0];
            let material = &self.sibeal_model.materials[0];
            self.texture_shader_pipeline.draw(
                &mut render_pass,
                mesh,
                &material.bind_group,
                &self.sibeal_instance_buffer,
                1,
            );

            // Draw the light object
            let mesh = &self.block_model.meshes[0];
            self.lighting_shader_pipeline
                .draw(&mut render_pass, mesh, 1);

            // Draw the targeted block highlight
            if let Some(block) = &player_target_block {
                // Create block instance
                let instance = wireframe::Instance {
                    model: block.to_instance().model,
                    color: [1., 1., 1.],
                };
                self.draw_context.queue.write_buffer(
                    &self.block_wireframe_instance_buffer,
                    0,
                    bytemuck::cast_slice(&[instance]),
                );

                self.wireframe_pipeline.draw(
                    &mut render_pass,
                    &self.block_wireframe_mesh,
                    &self.block_wireframe_instance_buffer,
                    1,
                );
            }
        }

        stopwatch.stamp_and_reset("Render pass");

        total_stopwatch.stamp_and_reset("Total render loop");

        stopwatch
            .get_debug_strings()
            .into_iter()
            .chain(total_stopwatch.get_debug_strings())
            .chain(counter.get_debug_strings())
            .chain([
                format!("pos: {:?}", self.camera.pos),
                // format!("player: {:#?}", game.player.aabb()),
                format!("Blocks rendered: {}", self.instances_cpu.len()),
                format!("Target block: {player_target_block:?}"),
            ])
            .for_each(|l| DEBUG_WINDOW.add_line(&l));

        self.ui.render(
            &self.draw_context,
            &mut encoder,
            &self.camera,
            &texture_view,
            game,
        );

        DEBUG_WINDOW.clear();

        // Actually run the operations on the GPU
        self.draw_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        // Show the new output to the screen
        output.present();
    }
}

impl Subscriber for RenderState {
    fn handle_message(&mut self, event: &Message) {
        use Message::*;
        match event {
            PlayerMoved((pos, orientation)) => {
                self.camera.pos.set(*pos);
                self.camera.yaw.set(orientation.yaw);
                self.camera.pitch.set(orientation.pitch);
                self.update_camera_buffer();
            }
            SetInteractionMode(mode) => match mode {
                InteractionMode::Game => {
                    if self.draw_context.grab_cursor().is_err() {
                        log::warn!("WARNING: Failed to grab cursor!");
                    }
                }
                InteractionMode::UI | InteractionMode::Block(_) => {
                    self.draw_context.ungrab_cursor();
                }
            },
            _ => (),
        }
    }
}
