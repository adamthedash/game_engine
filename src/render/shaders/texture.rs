use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferAddress, BufferBindingType,
    ColorTargetState, ColorWrites, DepthBiasState, DepthStencilState, Device, Face, FragmentState,
    FrontFace, MultisampleState, PipelineCompilationOptions, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPass,
    RenderPipelineDescriptor, SamplerBindingType, ShaderModule, ShaderModuleDescriptor,
    ShaderStages, StencilState, TextureFormat, TextureSampleType, TextureViewDimension,
    VertexBufferLayout, VertexState, VertexStepMode, vertex_attr_array,
};

use crate::render::{model::Mesh, texture::Texture};

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

/// GPU buffer version of Instance
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Instance {
    pub model: [[f32; 4]; 4],
    pub texture_index: u32,
    pub normal: [[f32; 3]; 3],
}

impl Instance {
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

/// A shader template without data buffers linked
pub struct TextureShaderPipelineLayout {
    pub texture_layout: BindGroupLayout,
    camera_layout: BindGroupLayout,
    lighting_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    shader: ShaderModule,
}

impl TextureShaderPipelineLayout {
    ///
    /// group 0: Texture (Fragment)
    ///     binding 0: Texture
    ///     binding 1: Sampler
    /// group 1: Camera (Vertex)
    ///     binding 0: Camera Uniform
    /// group 2: Lighting (Vertex + Fragment)
    ///     binding 0: Lighting Uniform
    ///
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./texture.wgsl").into()),
        });

        let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2Array,
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

        let camera_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        let lighting_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Lighting Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Texture Render Pipeline Layout"),
            bind_group_layouts: &[&texture_layout, &camera_layout, &lighting_layout],
            push_constant_ranges: &[],
        });

        Self {
            texture_layout,
            camera_layout,
            pipeline_layout,
            shader,
            lighting_layout,
        }
    }

    pub fn init(
        self,
        device: &Device,
        texture_format: &TextureFormat,
        camera_buffer: &Buffer,
        lighting_buffer: &Buffer,
    ) -> TextureShaderPipeline {
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &self.camera_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        let lighting_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Lighting Bind Group"),
            layout: &self.lighting_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: lighting_buffer.as_entire_binding(),
            }],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Texture Render Pipeline"),
            layout: Some(&self.pipeline_layout),
            // Vertex - the corners of the triangle
            vertex: VertexState {
                module: &self.shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::LAYOUT, Instance::LAYOUT],
                compilation_options: PipelineCompilationOptions::default(),
            },
            // Fragment - The inside of the triangle
            fragment: Some(FragmentState {
                module: &self.shader,
                entry_point: Some("fs_main"),
                // How the colour will be applied to the screen
                targets: &[Some(ColorTargetState {
                    format: *texture_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            // How the triangle will be created & displayed
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Cw,
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

        TextureShaderPipeline {
            layouts: self,
            camera_bind_group,
            render_pipeline: pipeline,
            lighting_bind_group,
        }
    }
}

/// An instantiated shader ready to go for rendering
pub struct TextureShaderPipeline {
    layouts: TextureShaderPipelineLayout,
    camera_bind_group: BindGroup,
    lighting_bind_group: BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl TextureShaderPipeline {
    pub fn draw(
        &self,
        render_pass: &mut RenderPass,
        mesh: &Mesh,
        texture_bind_group: &BindGroup,
        instance_buffer: &Buffer,
        num_instances: usize,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.set_bind_group(0, texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(2, &self.lighting_bind_group, &[]);

        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..num_instances as u32);
    }
}
