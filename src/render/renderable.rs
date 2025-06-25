use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

/// Holds all the info for a instanced renderable thing
pub struct Renderable {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub instance_buffer: Buffer,
    pub num_vertices: usize,
    pub num_indices: usize,
}

impl Renderable {
    pub fn new(
        device: &Device,
        name: &str,
        vertices: &[[f32; 3]],
        indices: &[u32],
        instance_buffer_size: u64,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("Vertex Buffer: {name:?}")),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("Index Buffer: {name:?}")),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX,
        });
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some(&format!("Instance Buffer: {name:?}")),
            size: instance_buffer_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            num_vertices: vertices.len(),
            num_indices: indices.len(),
        }
    }
}
