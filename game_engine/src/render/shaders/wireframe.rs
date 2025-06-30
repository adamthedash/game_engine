use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferAddress, BufferBindingType,
    ColorTargetState, ColorWrites, DepthBiasState, DepthStencilState, Device, FragmentState,
    FrontFace, MultisampleState, PipelineCompilationOptions, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPass,
    RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderStages, StencilState,
    TextureFormat, VertexBufferLayout, VertexState, VertexStepMode, vertex_attr_array,
};

use crate::render::{model::Mesh, texture::Texture};

/// Represents a vertex on the GPU
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3], // XYZ in NDC, CCW order
}

impl Vertex {
    /// Describes the memory layout of the vector buffer for the GPU
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x3,
        ],
    };
}

/// GPU buffer version of Instance
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Instance {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 3],
}

impl Instance {
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Instance,
        attributes: &vertex_attr_array![
            5 => Float32x4,
            6 => Float32x4,
            7 => Float32x4,
            8 => Float32x4,
            9 => Float32x3,
        ],
    };
}

/// A shader template without data buffers linked
pub struct WireframeShaderPipelineLayout {
    camera_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    shader: ShaderModule,
}

impl WireframeShaderPipelineLayout {
    ///
    /// group 0: Camera (Vertex)
    ///     binding 0: Camera Uniform
    ///
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Wireframe Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./wireframe.wgsl").into()),
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

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Wireframe Render Pipeline Layout"),
            bind_group_layouts: &[&camera_layout],
            push_constant_ranges: &[],
        });

        Self {
            camera_layout,
            pipeline_layout,
            shader,
        }
    }

    pub fn init(
        self,
        device: &Device,
        texture_format: &TextureFormat,
        camera_buffer: &Buffer,
    ) -> WireframeShaderPipeline {
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &self.camera_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Wireframe Render Pipeline"),
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
                topology: PrimitiveTopology::LineList,
                front_face: FrontFace::Cw,
                //cull_mode: Some(Face::Back),
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

        WireframeShaderPipeline {
            _layouts: self,
            camera_bind_group,
            render_pipeline: pipeline,
        }
    }
}

/// An instantiated shader ready to go for rendering
pub struct WireframeShaderPipeline {
    _layouts: WireframeShaderPipelineLayout,
    camera_bind_group: BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl WireframeShaderPipeline {
    pub fn draw(
        &self,
        render_pass: &mut RenderPass,
        mesh: &Mesh,
        instance_buffer: &Buffer,
        num_instance: usize,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..num_instance as u32);
    }
}
