use enum_map::Enum;
use num_derive::{FromPrimitive, ToPrimitive};

use crate::data::item::ItemType;

#[derive(Debug, Enum, PartialEq, Eq, Clone, Copy, ToPrimitive, FromPrimitive)]
pub enum BlockType {
    Air,
    Dirt,
    Stone,
    DarkStone,
    MossyStone,
    VoidStone,
    RadioactiveStone,
    // Ores
    Copper,
    Tin,
    Iron,
    Coal,
    MagicMetal,
}

#[derive(Debug, Clone)]
pub struct BlockData {
    pub(super) texture_path: &'static str,

    pub block_type: BlockType,
    /// None == unbreakable, 0 == broken by anything, bigger == harder to break
    pub hardness: Option<u32>,
    pub item_on_break: ItemType,
    pub renderable: bool,
}

pub(super) const TEXTURE_FOLDER: &str = "res/meshes";
pub(super) const BLOCK_DATA: [BlockData; 12] = [
    BlockData {
        block_type: BlockType::Air,
        hardness: None,
        item_on_break: ItemType::Dirt,
        texture_path: "dirt.png",
        renderable: false,
    },
    BlockData {
        block_type: BlockType::Dirt,
        hardness: Some(0),
        item_on_break: ItemType::Dirt,
        texture_path: "dirt.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::Stone,
        hardness: Some(10),
        item_on_break: ItemType::Stone,
        texture_path: "stone.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::DarkStone,
        hardness: Some(100),
        item_on_break: ItemType::DarkStone,
        texture_path: "darkstone.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::MossyStone,
        hardness: Some(10),
        item_on_break: ItemType::MossyStone,
        texture_path: "mossystone.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::VoidStone,
        hardness: Some(200),
        item_on_break: ItemType::VoidStone,
        texture_path: "voidstone.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::RadioactiveStone,
        hardness: Some(200),
        item_on_break: ItemType::RadioactiveStone,
        texture_path: "radioactivestone.png",
        renderable: true,
    },
    // Ores
    BlockData {
        block_type: BlockType::Copper,
        hardness: Some(100),
        item_on_break: ItemType::Copper,
        texture_path: "copper.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::Tin,
        hardness: Some(200),
        item_on_break: ItemType::Tin,
        texture_path: "tin.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::Iron,
        hardness: Some(300),
        item_on_break: ItemType::Iron,
        texture_path: "iron.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::Coal,
        hardness: Some(300),
        item_on_break: ItemType::Coal,
        texture_path: "coal.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::MagicMetal,
        hardness: Some(400),
        item_on_break: ItemType::MagicMetal,
        texture_path: "magic_metal.png",
        renderable: true,
    },
];
