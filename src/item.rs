use std::{path::Path, sync::OnceLock};

use egui::{ImageSource, ahash::HashMapExt, load::SizedTexture};
use egui_wgpu::Renderer;
use rustc_hash::FxHashMap;

use crate::render::{context::DrawContext, texture::Texture};

pub type ItemId = usize;

/// Static item information
#[derive(Debug)]
pub struct Item {
    pub id: ItemId,

    // Egui texture
    pub icon: Option<ImageSource<'static>>,
}

/// Global static item information, will be set when the renderer loads
pub static ITEMS: OnceLock<FxHashMap<ItemId, Item>> = OnceLock::new();

const ITEM_TEXTURES: [&str; 3] = [
    "res/icons/smiley.png", // ID = 0
    "res/icons/dirt.png",   // ID = 1
    "res/icons/stone.png",
];

/// Initialise item info
pub fn init_item_info(draw_context: &DrawContext, egui_renderer: &mut Renderer) {
    let items = ITEM_TEXTURES
        .iter()
        .map(|p| {
            let tex =
                Texture::from_image(Path::new(p), &draw_context.device, &draw_context.queue, p)
                    .unwrap();

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
        .enumerate()
        .fold(FxHashMap::new(), |mut hm, (id, texture)| {
            hm.insert(
                id,
                Item {
                    id,
                    icon: Some(texture),
                },
            );

            hm
        });

    ITEMS.set(items).unwrap();
}
