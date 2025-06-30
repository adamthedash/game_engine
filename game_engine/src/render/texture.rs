use std::path::Path;

use anyhow::Result;
use image::{EncodableLayout, ImageReader, RgbaImage};
use wgpu::{
    CompareFunction, Device, Extent3d, FilterMode, Origin3d, Queue, Sampler, SamplerDescriptor,
    SurfaceConfiguration, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    /// Creates a texture on the GPU from the given image file
    pub fn from_image(path: &Path, device: &Device, queue: &Queue, label: &str) -> Result<Self> {
        let image = ImageReader::open(path)?.decode()?.to_rgba8();

        let image_size = image.dimensions();
        let texture_size = Extent3d {
            width: image_size.0,
            height: image_size.1,
            depth_or_array_layers: 1,
        };
        // Set up the texture container (empty)
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(&format!("Texture: {label}")),
            size: texture_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        });
        // Wrte the image data to the texture
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &image,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image_size.0),
                rows_per_image: Some(image_size.1),
            },
            texture_size,
        );

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some(&format!("Sampler: {label}")),
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    /// Creates a texture array on the GPU from the given image files
    pub fn from_images(
        paths: &[&Path],
        device: &Device,
        queue: &Queue,
        label: &str,
    ) -> Result<Self> {
        let mut images = paths
            .iter()
            .map(|path| Ok(ImageReader::open(path)?.decode()?.to_rgba8()))
            .collect::<Result<Vec<_>>>()?;
        assert!(!images.is_empty(), "No images supplied.");

        let image_size = images.first().unwrap().dimensions();
        assert!(
            images.iter().all(|img| img.dimensions() == image_size),
            "All images must be the same dimensions"
        );

        // Append on an extra blank texture so we force 2dArray texture
        if images.len() == 1 {
            let dummy_img = RgbaImage::new(image_size.0, image_size.1);
            images.push(dummy_img);
        }

        let texture_size = Extent3d {
            width: image_size.0,
            height: image_size.1,
            depth_or_array_layers: images.len() as u32,
        };
        // Set up the texture container (empty)
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(&format!("Texture: {label}")),
            size: texture_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        });

        // Wrte the image data to the texture
        let image_bytes = images
            .iter()
            .flat_map(|img| img.as_bytes())
            .copied()
            .collect::<Vec<_>>();
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &image_bytes,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image_size.0),
                rows_per_image: Some(image_size.1),
            },
            texture_size,
        );

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some(&format!("Sampler: {label}")),
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub fn create_depth_texture(
        device: &Device,
        config: &SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(&format!("Texture: {label}")),
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some(&format!("Sampler: {label}")),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            lod_min_clamp: 0.,
            lod_max_clamp: 100.,
            compare: Some(CompareFunction::LessEqual),
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}
