use std::{path::PathBuf, str::FromStr, sync::Arc};

use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, Quaternion, Vector3};
use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferAddress, BufferBindingType, BufferUsages, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, DepthBiasState, DepthStencilState, Device, DeviceDescriptor, Face,
    Features, FragmentState, FrontFace, IndexFormat, InstanceDescriptor, LoadOp, MultisampleState,
    Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    SamplerBindingType, ShaderModuleDescriptor, ShaderStages, StencilState, StoreOp, Surface,
    SurfaceConfiguration, TextureSampleType, TextureViewDescriptor, TextureViewDimension,
    VertexBufferLayout, VertexState, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array,
};
use wgpu_text::{
    BrushBuilder, TextBrush,
    glyph_brush::{self, Text, ab_glyph::FontRef},
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    block::{BLOCK_INDICES, BLOCK_VERTICES, Block},
    camera::{Camera, CameraUniform},
    texture::Texture,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],       // XYZ in NDC, CCW order
    pub texture_coords: [f32; 2], // XY, origin top-left
}

impl Vertex {
    /// Describes the memory layout of the vector buffer for the GPU
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2,
        ],
    };
}

/// Represents a copy of a renderable thing, at a specific location
#[derive(Debug)]
pub struct Instance {
    pub pos: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.pos) * Matrix4::from(self.rotation)).into(),
        }
    }
}

/// GPU buffer version of Instance
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Instance,
        attributes: &vertex_attr_array![
            5 => Float32x4,
            6 => Float32x4,
            7 => Float32x4,
            8 => Float32x4,
        ],
    };
}

/// Holds all of the stuff related to rendering the game window.
pub struct RenderState<'a> {
    pub window: Arc<Window>,
    surface: Surface<'static>,
    gpu_handle: wgpu::Instance,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    // Shader stuff
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
    // Texture stuff
    texture_bind_group: BindGroup,
    depth_texture: Texture,
    // Camera stuff
    pub camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    // Text stuff
    brush: TextBrush<FontRef<'a>>,
}

impl RenderState<'_> {
    pub async fn new(window: Arc<Window>, camera: Camera) -> Self {
        let (surface, gpu_handle, device, queue, config) =
            RenderState::init_gpu(window.clone()).await;

        // Shaders
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/shader.wgsl").into()),
        });
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(BLOCK_VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(BLOCK_INDICES),
            usage: BufferUsages::INDEX,
        });

        // Texture
        let (texture_bind_group_layout, texture_bind_group, depth_texture) =
            RenderState::init_texture(&device, &queue, &config);

        // Camera
        let (camera_uniform, camera_buffer, camera_bind_group_layout, camera_bind_group) =
            RenderState::init_camera(&device);

        // Render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            // Vertex - the corners of the triangle
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::LAYOUT, InstanceRaw::LAYOUT],
                compilation_options: PipelineCompilationOptions::default(),
            },
            // Fragment - The inside of the triangle
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                // How the colour will be applied to the screen
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            // How the triangle will be created & displayed
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            // Rest of stuff is just defaults
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Text
        let brush = BrushBuilder::using_font_bytes(include_bytes!(
            "../res/fonts/JetBrainsMono-Regular.ttf"
        ))
        .unwrap()
        .build(&device, config.width, config.height, config.format);

        // Instances - 4x4 grid of blocks
        let blocks = (0..16)
            .flat_map(|z| {
                (0..16).flat_map(move |x| {
                    (0..16).map(move |y| Block {
                        world_pos: (x, y, z),
                    })
                })
            })
            .flat_map(|b| b.to_instances())
            .collect::<Vec<_>>();

        let instance_data = blocks.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: BufferUsages::VERTEX,
        });

        Self {
            window,
            surface,
            gpu_handle,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: BLOCK_INDICES.len() as u32,
            texture_bind_group,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            brush,
            instances: blocks,
            instance_buffer,
            depth_texture,
        }
    }

    fn init_camera(device: &Device) -> (CameraUniform, Buffer, BindGroupLayout, BindGroup) {
        // Camera - probably shouldn't be here, we want these values somewhere
        let camera_uniform = CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        (
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        )
    }

    fn init_texture(
        device: &Device,
        queue: &Queue,
        config: &SurfaceConfiguration,
    ) -> (BindGroupLayout, BindGroup, Texture) {
        // Textures
        let texture = Texture::from_image(
            &PathBuf::from_str("res/images/image.png").unwrap(),
            device,
            queue,
            "smiley",
        )
        .expect("Failed to load texture");

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Binding Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        // Depth
        let depth_texture = Texture::create_depth_texture(device, config, "Depth Texture");

        (texture_bind_group_layout, texture_bind_group, depth_texture)
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
            .request_device(&DeviceDescriptor {
                required_features: Features::empty(),
                ..Default::default()
            })
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
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.height = size.height;
        self.config.width = size.width;
        self.surface.configure(&self.device, &self.config);

        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.config, "Depth Texture");

        self.camera.aspect = size.width as f32 / size.height as f32;
        self.update_camera_buffer();

        self.brush
            .resize_view(size.width as f32, size.height as f32, &self.queue);
    }

    /// Syncs the camera state to the buffer being rendered in the GPU
    pub fn update_camera_buffer(&mut self) {
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    /// Perform the actual rendering to the screen
    pub fn render(&mut self) {
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
            // Clear the screen and fill it with grey
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
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

            // Render the triangle with the shader
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as u32);
        }

        // Text needs a separate render pass because it doesn't like the depth buffer
        let camera_debug_text = format!("{:#?}", self.camera);
        let text = glyph_brush::Section::default().add_text(Text::new(&camera_debug_text));
        self.brush
            .queue(&self.device, &self.queue, [&text])
            .unwrap();
        {
            // Clear the screen and fill it with grey
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            self.brush.draw(&mut render_pass);
        }

        // Actually run the operations on the GPU
        self.queue.submit(std::iter::once(encoder.finish()));

        // Show the new output to the screen
        output.present();
    }
}
