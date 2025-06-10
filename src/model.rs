use std::{
    ops::Range,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{render::Vertex, texture::Texture};
use anyhow::{Context, Result};
use tobj::{LoadOptions, load_obj};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, BufferUsages, Device,
    Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub struct Material {
    pub name: String,
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

impl Model {
    /// Load a model from an OBJ file, optionally with a material
    pub fn load_model(
        path: &Path,
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
    ) -> Result<Self> {
        // Load .obj file
        let (models, materials) = load_obj(
            path,
            &LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )?;

        // Load materials onto GPU as textures
        let materials = materials?
            .into_iter()
            .map(|m| -> Result<_> {
                let texture_filename =
                    PathBuf::from_str(&m.diffuse_texture.expect("No diffuse texture"))?;
                // Assume materials are stored alongside obj
                let texture_path = path.parent().unwrap().join(texture_filename);
                let texture = Texture::from_image(&texture_path, device, queue, &m.name)
                    .with_context(|| format!("Failed to load texture: {:?}", texture_path))?;

                let bind_group = device.create_bind_group(&BindGroupDescriptor {
                    layout,
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
                    label: Some(&format!("Bind group: {}", m.name)),
                });

                Ok(Material {
                    name: m.name,
                    texture,
                    bind_group,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        // Load meshes into buffers
        let meshes = models
            .into_iter()
            .map(|m| {
                let positions = m.mesh.positions.chunks_exact(3);
                let tex_coords = m
                    .mesh
                    .texcoords
                    .chunks_exact(2)
                    .map(|tc| [tc[0], 1. - tc[1]]);

                let normals: Box<dyn Iterator<Item = [f32; 3]>> = if m.mesh.normals.is_empty() {
                    Box::new(std::iter::repeat([0_f32, 0., 0.]))
                } else {
                    Box::new(
                        m.mesh
                            .normals
                            .chunks_exact(3)
                            .map(|c| c.try_into().expect("Bad chunk")),
                    )
                };

                let vertices = positions
                    .zip(tex_coords)
                    .zip(normals)
                    .map(|((p, tc), n)| Vertex {
                        position: p.try_into().unwrap(),
                        texture_coords: tc,
                        normals: n,
                    })
                    .collect::<Vec<_>>();
                println!("verts: {}", vertices.len());

                let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some(&format!("Vertex Buffer: {:?}", path)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some(&format!("Index Buffer: {:?}", path)),
                    contents: bytemuck::cast_slice(&m.mesh.indices),
                    usage: BufferUsages::INDEX,
                });

                Mesh {
                    name: path.to_str().unwrap().to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: m.mesh.indices.len() as u32,
                    material: m.mesh.material_id.unwrap_or(0),
                }
            })
            .collect::<Vec<_>>();

        Ok(Model { meshes, materials })
    }
}

/// Helper trait to put drawing logic onto RenderPass instead of Model
pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}
