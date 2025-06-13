use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, ColorTargetState,
    ColorWrites, DepthBiasState, DepthStencilState, Device, Face, FragmentState, FrontFace,
    MultisampleState, PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor,
    PrimitiveState, PrimitiveTopology, RenderPass, RenderPipelineDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderStages, StencilState, TextureFormat, VertexState,
};

use crate::render::{state::Vertex, texture::Texture};

/// A shader template without data buffers linked
pub struct LightingShaderPipelineLayout {
    camera_layout: BindGroupLayout,
    lighting_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    shader: ShaderModule,
}

impl LightingShaderPipelineLayout {
    ///
    /// group 0: Camera (Vertex)
    ///     binding 0: Camera Uniform
    /// group 1: Lighting (Vertex + Fragment)
    ///     binding 0: Lighting Uniform
    ///
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Light Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./lighting.wgsl").into()),
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
            label: Some("Lighting Render Pipeline Layout"),
            bind_group_layouts: &[&camera_layout, &lighting_layout],
            push_constant_ranges: &[],
        });

        Self {
            camera_layout,
            lighting_layout,
            pipeline_layout,
            shader,
        }
    }

    pub fn init(
        self,
        device: &Device,
        texture_format: &TextureFormat,
        camera_buffer: &Buffer,
        lighting_buffer: &Buffer,
    ) -> LightingShaderPipeline {
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
            label: Some("Lighting Render Pipeline"),
            layout: Some(&self.pipeline_layout),
            // Vertex - the corners of the triangle
            vertex: VertexState {
                module: &self.shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::LAYOUT],
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

        LightingShaderPipeline {
            layouts: self,
            camera_bind_group,
            render_pipeline: pipeline,
            lighting_bind_group,
        }
    }
}

/// An instantiated shader ready to go for rendering
pub struct LightingShaderPipeline {
    layouts: LightingShaderPipelineLayout,
    camera_bind_group: BindGroup,
    lighting_bind_group: BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl LightingShaderPipeline {
    /// Loads the data buffers into the right slots in the GPU
    pub fn setup_rendering_pass(
        &self,
        render_pass: &mut RenderPass,
        vertex_buffer: &Buffer,
        index_buffer: &Buffer,
    ) {
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.lighting_bind_group, &[]);
    }
}
