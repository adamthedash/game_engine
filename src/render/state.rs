use std::{path::Path, sync::Arc, time};

use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix3, Matrix4, Quaternion, Vector3};
use egui_wgpu::ScreenDescriptor;
use wgpu::{
    Backends, Buffer, BufferAddress, BufferDescriptor, BufferUsages, CommandEncoder,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Features, InstanceDescriptor, LoadOp,
    Operations, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration,
    TextureFormat, TextureView, TextureViewDescriptor, VertexBufferLayout, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    error::ExternalError,
    window::{CursorGrabMode, Window},
};

use crate::{
    block::Block,
    camera::{Camera, CameraUniform},
    game::GameState,
    render::{
        light::LightingUniform,
        model::Model,
        shaders::{
            lighting::{LightingShaderPipeline, LightingShaderPipelineLayout},
            texture::{TextureShaderPipeline, TextureShaderPipelineLayout},
        },
        texture::Texture,
    },
    world::{BlockType, Chunk, ChunkPos},
};

/// Represents a vertex on the GPU
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],       // XYZ in NDC, CCW order
    pub texture_coords: [f32; 2], // XY, origin top-left
    pub normals: [f32; 3],
}

impl Vertex {
    /// Describes the memory layout of the vector buffer for the GPU
    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2,
            2 => Float32x3,
        ],
    };
}

/// Represents a copy of a renderable thing, at a specific location
#[derive(Debug)]
pub struct Instance {
    pub pos: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub texture_index: u32,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.pos) * Matrix4::from(self.rotation)).into(),
            texture_index: self.texture_index,
            normal: Matrix3::from(self.rotation).into(),
        }
    }
}

/// GPU buffer version of Instance
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    texture_index: u32,
    normal: [[f32; 3]; 3],
}

impl InstanceRaw {
    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Instance,
        attributes: &vertex_attr_array![
            5 => Float32x4,
            6 => Float32x4,
            7 => Float32x4,
            8 => Float32x4,
            9 => Uint32,
            10 => Float32x3,
            11 => Float32x3,
            12 => Float32x3,
        ],
    };
}

/// Maximum number of block instances we can render at once
const INSTANCE_BUFFER_MAX_SIZE: u64 =
    (std::mem::size_of::<InstanceRaw>() * Chunk::BLOCKS_PER_CHUNK * 512) as u64;

/// Holds all of the stuff related to rendering the game window.
pub struct RenderState {
    pub window: Arc<Window>,
    surface: Surface<'static>,
    gpu_handle: wgpu::Instance,
    device: Device,
    queue: Queue,
    pub config: SurfaceConfiguration,
    // Shader stuff
    texture_shader_pipeline: TextureShaderPipeline,
    instance_buffer: Buffer,
    // Texture stuff
    obj_model: Model,
    depth_texture: Texture,
    // Camera stuff
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    // Lighting stuff
    lighting_uniform: LightingUniform,
    lighting_buffer: Buffer,
    lighting_shader_pipeline: LightingShaderPipeline,
    // GUI stuff
    egui_state: egui_winit::State,
    egui_context: egui::Context,
    egui_renderer: egui_wgpu::Renderer,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let (surface, gpu_handle, device, queue, config) =
            RenderState::init_gpu(window.clone()).await;

        // Shaders
        let texture_shader = TextureShaderPipelineLayout::new(&device);
        let light_shader = LightingShaderPipelineLayout::new(&device);

        // Texture
        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");

        // Mesh
        let obj_path = Path::new(env!("OUT_DIR")).join("res/meshes/block.obj");
        let obj_model =
            Model::load_model(&obj_path, &device, &queue, &texture_shader.texture_layout).unwrap();

        // Camera
        let (camera_uniform, camera_buffer) = RenderState::init_camera(&device);
        let (lighting_uniform, lighting_buffer) = RenderState::init_lighting(&device);

        // Render Pipeline
        let texture_shader_pipeline =
            texture_shader.init(&device, &config.format, &camera_buffer, &lighting_buffer);
        let lighting_shader_pipeline =
            light_shader.init(&device, &config.format, &camera_buffer, &lighting_buffer);

        // Instances of blocks
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: INSTANCE_BUFFER_MAX_SIZE,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // GUI
        let egui_renderer =
            egui_wgpu::Renderer::new(&device, TextureFormat::Bgra8UnormSrgb, None, 1, false);
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
            window,
            surface,
            gpu_handle,
            device,
            queue,
            config,
            texture_shader_pipeline,
            obj_model,
            camera_uniform,
            camera_buffer,
            instance_buffer,
            depth_texture,
            lighting_uniform,
            lighting_buffer,
            lighting_shader_pipeline,
            egui_state,
            egui_context,
            egui_renderer,
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

    async fn init_gpu<'a>(
        window: Arc<Window>,
    ) -> (
        Surface<'a>,
        wgpu::Instance,
        Device,
        Queue,
        SurfaceConfiguration,
    ) {
        // Get handle to GPU
        let gpu_handle = wgpu::Instance::new(&InstanceDescriptor {
            backends: Backends::VULKAN,
            ..Default::default()
        });

        // Create a texture surface - this is what we draw on
        let surface = gpu_handle.create_surface(window.clone()).unwrap();
        // Adapter is another kind of handle to GPU
        let adapter = gpu_handle
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // Device - Yet another GPU handle
        // Queue - Used to send draw operations to the GPU
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        // Configure the draw surface to match the window
        let window_size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, window_size.width, window_size.height)
            .unwrap();
        surface.configure(&device, &config);

        (surface, gpu_handle, device, queue, config)
    }

    /// Make sure draw surface and window are tied together
    pub fn resize(&mut self, size: PhysicalSize<u32>, camera: &mut Camera) {
        self.config.height = size.height;
        self.config.width = size.width;
        self.surface.configure(&self.device, &self.config);

        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

        camera.aspect = size.width as f32 / size.height as f32;
        self.update_camera_buffer(camera);
    }

    /// Syncs the camera state to the buffer being rendered in the GPU
    pub fn update_camera_buffer(&mut self, camera: &Camera) {
        self.camera_uniform.update_view_proj(camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    /// Perform the actual rendering to the screen
    pub fn render(&mut self, game: &GameState) {
        let start_time = time::Instant::now();

        let player_target_block = game.get_player_target_block();

        // Check what blocks are candidates for rendering
        let (player_chunk, _) = game.player.camera.pos.to_block_pos().to_chunk_offset();
        let player_vision_chunks =
            (game.player.camera.zfar as u32).div_ceil(Chunk::CHUNK_SIZE as u32);
        let visible_blocks = player_chunk
            // Only render chunks within vision distance of the player (plus 1 chunk buffer)
            .chunks_within(player_vision_chunks + 1)
            // Only render chunks in the player's viewport
            .filter(|chunk_pos| {
                Chunk::CORNER_OFFSETS.iter().any(|o| {
                    let chunk_corner = ChunkPos(chunk_pos.0 + o);
                    game.player
                        .camera
                        .in_view(&chunk_corner.to_block_pos().to_world_pos())
                })
            })
            .flat_map(|pos| game.world.chunks.get(&pos))
            .flat_map(|chunk| {
                chunk
                    .iter_blocks()
                    // Don't render air blocks
                    .filter(|b| b.block_type != BlockType::Air)
                    // Only render exposed blocks
                    .filter(|b| {
                        let (_, block_pos) = b.block_pos.to_chunk_offset();
                        chunk.is_block_exposed(block_pos)
                    })
            })
            // If we're targetting the block, change the texture
            .map(|b| {
                if let Some(target_block) = &player_target_block
                    && b.block_pos.0 == target_block.block_pos.0
                {
                    Block {
                        block_type: BlockType::Smiley,
                        ..b
                    }
                } else {
                    b
                }
            });

        // Convert blocks to renderable instances
        let instances = visible_blocks
            .map(|block| block.to_instance().to_raw())
            .collect::<Vec<_>>();
        self.queue
            .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

        // Get a view on the surface texture that we'll draw to
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        // Encoder is used to send operations to the GPU queue
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            // Clear the screen and fill it with blue
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
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
            render_pass.set_pipeline(&self.texture_shader_pipeline.render_pipeline);

            let mesh = &self.obj_model.meshes[0];
            let material = &self.obj_model.materials[0];
            self.texture_shader_pipeline.setup_rendering_pass(
                &mut render_pass,
                &mesh.vertex_buffer,
                &mesh.index_buffer,
                &material.bind_group,
            );

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw_indexed(0..mesh.num_elements, 0, 0..instances.len() as u32);

            // Draw the light object
            render_pass.set_pipeline(&self.lighting_shader_pipeline.render_pipeline);
            self.lighting_shader_pipeline.setup_rendering_pass(
                &mut render_pass,
                &mesh.vertex_buffer,
                &mesh.index_buffer,
            );
            render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }

        let end_time = time::Instant::now();
        let time_taken = end_time.duration_since(start_time);

        let debug_text = [
            format!("{:#?}", game.player.camera),
            format!(
                "Render pass: {:?} ({} FPS)",
                time_taken,
                1000. / time_taken.as_millis() as f32
            ),
            format!("Blocks rendered: {}", instances.len()),
            format!("Target block: {player_target_block:?}"),
        ];
        self.debug_render(&debug_text, &mut encoder, &view);

        // Actually run the operations on the GPU
        self.queue.submit(std::iter::once(encoder.finish()));

        // Show the new output to the screen
        output.present();
    }

    /// Displays the debug text overlay
    fn debug_render(&mut self, texts: &[String], encoder: &mut CommandEncoder, view: &TextureView) {
        let inputs = egui::RawInput::default();
        let output = self.egui_context.run(inputs, |ctx| {
            // UI code here
            egui::Window::new("Inventory").show(ctx, |ui| {
                texts.iter().for_each(|t| {
                    ui.label(t);
                });
            });
        });

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
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
                self.egui_renderer
                    .update_texture(&self.device, &self.queue, *id, image_delta);
            });

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
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

    /// Grab cursor control so the camera can be moved around
    pub fn grab_cursor(&self) -> Result<(), ExternalError> {
        self.window.set_cursor_grab(CursorGrabMode::Confined)?;
        self.window.set_cursor_visible(false);

        // Centre the cursor in the window
        self.centre_cursor()
    }

    /// Unlock the cursor so the player can interact with UI
    pub fn ungrab_cursor(&self) {
        self.window.set_cursor_grab(CursorGrabMode::None).unwrap();
        self.window.set_cursor_visible(true);
    }

    /// Set the cursor's position, position is in pixel space
    pub fn set_cursor_pos(&self, position: &PhysicalPosition<u32>) -> Result<(), ExternalError> {
        // On Wayland we need to lock it first
        self.window.set_cursor_grab(CursorGrabMode::Locked)?;
        self.window.set_cursor_position(*position)?;
        self.window.set_cursor_grab(CursorGrabMode::Confined)?;
        Ok(())
    }

    /// Centre the cursor on the screen
    pub fn centre_cursor(&self) -> Result<(), ExternalError> {
        self.set_cursor_pos(&PhysicalPosition::new(
            self.config.width / 2,
            self.config.height / 2,
        ))
    }
}
