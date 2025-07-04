use enum_map::Enum;

use crate::data::block::BlockType;

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ItemType {
    Dirt,
    Stone,
    DarkStone,
    MossyStone,
    VoidStone,
    RadioactiveStone,
}

// User-defined
pub(super) struct ItemData {
    pub item_type: ItemType,
    pub name: &'static str,
    pub icon_path: &'static str,
    pub weight: f32,
    pub block: Option<BlockType>,
}

pub(super) const ICON_PATH: &str = "res/icons";
pub(super) const ITEM_DATA: [ItemData; 6] = [
    ItemData {
        item_type: ItemType::Dirt,
        name: "Dirt",
        icon_path: "dirt.png",
        weight: 1.,
        block: Some(BlockType::Dirt),
    },
    ItemData {
        item_type: ItemType::Stone,
        name: "Stone",
        icon_path: "stone.png",
        weight: 1.,
        block: Some(BlockType::Stone),
    },
    ItemData {
        item_type: ItemType::DarkStone,
        name: "Dark Stone",
        icon_path: "darkstone.png",
        weight: 1.,
        block: Some(BlockType::DarkStone),
    },
    ItemData {
        item_type: ItemType::MossyStone,
        name: "Mossy Stone",
        icon_path: "mossystone.png",
        weight: 1.,
        block: Some(BlockType::MossyStone),
    },
    ItemData {
        item_type: ItemType::VoidStone,
        name: "Void Stone",
        icon_path: "voidstone.png",
        weight: 1.,
        block: Some(BlockType::VoidStone),
    },
    ItemData {
        item_type: ItemType::RadioactiveStone,
        name: "Radioactive Stone",
        icon_path: "radioactivestone.png",
        weight: 1.,
        block: Some(BlockType::RadioactiveStone),
    },
];
