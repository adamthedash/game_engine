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
}

pub(super) struct BlockData {
    pub block_type: BlockType,
    pub breakable: bool,
    pub item_on_break: ItemType,
    pub texture_path: &'static str,
    pub renderable: bool,
}

pub(super) const TEXTURE_FOLDER: &str = "res/meshes";
pub(super) const BLOCK_DATA: [BlockData; 7] = [
    BlockData {
        block_type: BlockType::Air,
        breakable: false,
        item_on_break: ItemType::Dirt,
        texture_path: "dirt.png",
        renderable: false,
    },
    BlockData {
        block_type: BlockType::Dirt,
        breakable: true,
        item_on_break: ItemType::Dirt,
        texture_path: "dirt.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::Stone,
        breakable: true,
        item_on_break: ItemType::Stone,
        texture_path: "stone.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::DarkStone,
        breakable: true,
        item_on_break: ItemType::DarkStone,
        texture_path: "darkstone.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::MossyStone,
        breakable: true,
        item_on_break: ItemType::MossyStone,
        texture_path: "mossystone.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::VoidStone,
        breakable: true,
        item_on_break: ItemType::VoidStone,
        texture_path: "voidstone.png",
        renderable: true,
    },
    BlockData {
        block_type: BlockType::RadioactiveStone,
        breakable: true,
        item_on_break: ItemType::RadioactiveStone,
        texture_path: "radioactivestone.png",
        renderable: true,
    },
];
