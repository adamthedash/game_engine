use std::{path::Path, sync::Arc, time};

use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix3, Matrix4, Quaternion, Vector3};
use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Device,
    LoadOp, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, StoreOp, VertexBufferLayout, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    block::Block,
    camera::{Camera, CameraUniform},
    game::GameState,
    render::{
        context::DrawContext,
        light::LightingUniform,
        model::Model,
        shaders::{
            lighting::{LightingShaderPipeline, LightingShaderPipelineLayout},
            texture::{TextureShaderPipeline, TextureShaderPipelineLayout},
        },
        texture::Texture,
        ui::UI,
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
    // GPU and window stuff
    pub draw_context: DrawContext,
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
    pub ui: UI,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let draw_context = DrawContext::new(window).await;

        // Shaders
        let texture_shader = TextureShaderPipelineLayout::new(&draw_context.device);
        let light_shader = LightingShaderPipelineLayout::new(&draw_context.device);

        // Texture
        let depth_texture = Texture::create_depth_texture(
            &draw_context.device,
            &draw_context.config,
            "Depth Texture",
        );

        // Mesh
        let obj_path = Path::new(env!("OUT_DIR")).join("res/meshes/block.obj");
        let obj_model = Model::load_model(
            &obj_path,
            &draw_context.device,
            &draw_context.queue,
            &texture_shader.texture_layout,
        )
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

        // Instances of blocks
        let instance_buffer = draw_context.device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: INSTANCE_BUFFER_MAX_SIZE,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // GUI
        let ui = UI::new(&draw_context.device, &draw_context.window);

        Self {
            draw_context,
            texture_shader_pipeline,
            obj_model,
            camera_uniform,
            camera_buffer,
            instance_buffer,
            depth_texture,
            lighting_uniform,
            lighting_buffer,
            lighting_shader_pipeline,
            ui,
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
    pub fn resize(&mut self, size: PhysicalSize<u32>, camera: &mut Camera) {
        self.draw_context.resize(size);
        self.depth_texture = Texture::create_depth_texture(
            &self.draw_context.device,
            &self.draw_context.config,
            "depth_texture",
        );

        camera.aspect = size.width as f32 / size.height as f32;
        self.update_camera_buffer(camera);
    }

    /// Syncs the camera state to the buffer being rendered in the GPU
    pub fn update_camera_buffer(&mut self, camera: &Camera) {
        self.camera_uniform.update_view_proj(camera);
        self.draw_context.queue.write_buffer(
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
        self.draw_context.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instances),
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
        self.ui
            .render(&self.draw_context, &mut encoder, &texture_view, &debug_text);

        // Actually run the operations on the GPU
        self.draw_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        // Show the new output to the screen
        output.present();
    }
}
