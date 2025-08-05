use std::{path::Path, sync::OnceLock};

use anyhow::Context;
use egui::{ImageSource, load::SizedTexture};
use egui_wgpu::Renderer;
use enum_map::EnumMap;

use crate::{
    data::{
        block::{self, BLOCK_DATA, BlockType, TEXTURE_FOLDER},
        item::{self, ICON_PATH, ITEM_DATA, ItemType},
    },
    render::{context::DrawContext, texture::Texture},
};

/// Instantiated item data
#[derive(Debug, Clone)]
pub struct ItemData {
    pub texture: ImageSource<'static>,
    pub data: item::ItemData,
}

/// Global static item information, will be set when the renderer loads
pub static ITEMS: OnceLock<EnumMap<ItemType, ItemData>> = OnceLock::new();

/// Initialise item info
pub fn init_item_info(draw_context: &DrawContext, egui_renderer: &mut Renderer) {
    let icon_folder = Path::new(ICON_PATH);
    let icons = ITEM_DATA
        .iter()
        .map(|p| {
            let path = icon_folder.join(p.icon_path);
            let tex = Texture::from_image(
                &path,
                &draw_context.device,
                &draw_context.queue,
                p.icon_path,
            )
            .unwrap_or_else(|_| panic!("Failed to load texture: {path:?}"));

            let texture_id = egui_renderer.register_native_texture(
                &draw_context.device,
                &tex.view,
                wgpu::FilterMode::Nearest,
            );

            ImageSource::Texture(SizedTexture::new(
                texture_id,
                [tex.texture.width() as f32, tex.texture.height() as f32],
            ))
        })
        .collect::<Vec<_>>();

    let map = ITEM_DATA
        .iter()
        .zip(icons)
        .map(|(d, icon)| ItemData {
            texture: icon,
            data: d.clone(),
        })
        .collect::<Vec<_>>();

    let map = EnumMap::from_fn(|k: ItemType| {
        map.iter()
            .find(|x| x.data.item_type == k)
            .unwrap_or_else(|| panic!("User-defiend ItemData not found for: {k:?}"))
            .clone()
    });

    ITEMS.set(map).unwrap();
}

#[derive(Debug, Clone)]
pub struct BlockData {
    /// Textures are loaded separately as a 3D texture array. This indexes into it.
    pub texture_index: u32,
    pub data: block::BlockData,
}

/// Global static item information, will be set when the renderer loads
pub static BLOCKS: OnceLock<EnumMap<BlockType, BlockData>> = OnceLock::new();
pub static BLOCK_TEXTURES: OnceLock<Texture> = OnceLock::new();

/// Initialise block info
pub fn init_block_info(draw_context: &DrawContext) {
    let texture_folder = Path::new(TEXTURE_FOLDER);
    let texture_paths = BLOCK_DATA
        .iter()
        .map(|b| texture_folder.join(b.texture_path))
        .collect::<Vec<_>>();
    let texture_paths = texture_paths
        .iter()
        .map(|p| p.as_path())
        .collect::<Vec<_>>();
    let textures = Texture::from_images(
        &texture_paths,
        &draw_context.device,
        &draw_context.queue,
        "Block Textures",
    )
    .context("Failed to load block textures.")
    .unwrap();

    let block_data = BLOCK_DATA
        .iter()
        .enumerate()
        .map(|(i, b)| BlockData {
            texture_index: i as u32,
            data: b.clone(),
        })
        .collect::<Vec<_>>();

    let map = EnumMap::from_fn(|k| {
        block_data
            .iter()
            .find(|x| x.data.block_type == k)
            .unwrap()
            .clone()
    });

    BLOCKS.set(map).unwrap();
    BLOCK_TEXTURES.set(textures).unwrap();
}
