use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Context, Result};
use tobj::{LoadOptions, load_obj};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, BufferUsages, Device,
    Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{render::Vertex, texture::Texture};

/// Represents a single model mesh / material. Eg. a single block type
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
        let texture_paths = materials?
            .into_iter()
            .map(|m| -> Result<_> {
                let texture_filename =
                    PathBuf::from_str(&m.diffuse_texture.expect("No diffuse texture"))?;
                // Assume materials are stored alongside obj
                let texture_path = path.parent().unwrap().join(texture_filename);
                Ok(texture_path)
            })
            .collect::<Result<Vec<_>>>()?;

        let texture = Texture::from_images(
            &texture_paths
                .iter()
                .map(|p| p.as_path())
                .collect::<Vec<_>>(),
            device,
            queue,
            &format!("Texture: {:?}", path.file_name()),
        )?;
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
            label: Some(&format!("Bind group: {:?}", path.file_name())),
        });

        let material = Material {
            name: path.file_name().unwrap().to_str().unwrap().to_string(),
            texture,
            bind_group,
        };

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
                }
            })
            .collect::<Vec<_>>();

        Ok(Model {
            meshes,
            materials: vec![material],
        })
    }
}
