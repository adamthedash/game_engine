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
    // Ores
    Copper,
    Tin,
    Bronze,
    Coal,
    Iron,
    Steel,
    MagicMetal,
    // Tools
    CopperPickaxe,
    BronzePickaxe,
    IronPickaxe,
    SteelPickaxe,
    MagicMetalPickaxe,
}

// User-defined
#[derive(Clone, Debug)]
pub struct ItemData {
    pub(super) icon_path: &'static str,

    pub item_type: ItemType,
    pub name: &'static str,
    pub weight: f32,
    pub block: Option<BlockType>,
    pub breaking_strength: Option<u32>,
}

pub(super) const ICON_PATH: &str = "res/icons";
pub(super) const ITEM_DATA: [ItemData; 18] = [
    ItemData {
        item_type: ItemType::Dirt,
        name: "Dirt",
        icon_path: "dirt.png",
        weight: 1.,
        block: Some(BlockType::Dirt),
        breaking_strength: None,
    },
    ItemData {
        item_type: ItemType::Stone,
        name: "Stone",
        icon_path: "stone.png",
        weight: 1.,
        block: Some(BlockType::Stone),
        breaking_strength: None,
    },
    ItemData {
        item_type: ItemType::DarkStone,
        name: "Dark Stone",
        icon_path: "darkstone.png",
        weight: 1.,
        block: Some(BlockType::DarkStone),
        breaking_strength: None,
    },
    ItemData {
        item_type: ItemType::MossyStone,
        name: "Mossy Stone",
        icon_path: "mossystone.png",
        weight: 1.,
        block: Some(BlockType::MossyStone),
        breaking_strength: None,
    },
    ItemData {
        item_type: ItemType::VoidStone,
        name: "Void Stone",
        icon_path: "voidstone.png",
        weight: 1.,
        block: Some(BlockType::VoidStone),
        breaking_strength: None,
    },
    ItemData {
        item_type: ItemType::RadioactiveStone,
        name: "Radioactive Stone",
        icon_path: "radioactivestone.png",
        weight: 1.,
        block: Some(BlockType::RadioactiveStone),
        breaking_strength: None,
    },
    // Ores
    ItemData {
        item_type: ItemType::Copper,
        name: "Copper Ore",
        icon_path: "copper.png",
        weight: 1.,
        block: Some(BlockType::Copper),
        breaking_strength: None,
    },
    ItemData {
        item_type: ItemType::Tin,
        name: "Tin Ore",
        icon_path: "tin.png",
        weight: 1.,
        block: Some(BlockType::Tin),
        breaking_strength: None,
    },
    ItemData {
        item_type: ItemType::Bronze,
        name: "Bronze Bar",
        icon_path: "bronze.png",
        weight: 1.,
        block: None,
        breaking_strength: None,
    },
    ItemData {
        item_type: ItemType::Coal,
        name: "Coal",
        icon_path: "coal.png",
        weight: 1.,
        block: Some(BlockType::Coal),
        breaking_strength: None,
    },
    ItemData {
        item_type: ItemType::Iron,
        name: "Iron Ore",
        icon_path: "iron.png",
        weight: 1.,
        block: Some(BlockType::Iron),
        breaking_strength: None,
    },
    ItemData {
        item_type: ItemType::Steel,
        name: "Steel Bar",
        icon_path: "steel.png",
        weight: 1.,
        block: None,
        breaking_strength: None,
    },
    ItemData {
        item_type: ItemType::MagicMetal,
        name: "Magic Metal Bar",
        icon_path: "magic_metal.png",
        weight: 1.,
        block: Some(BlockType::MagicMetal),
        breaking_strength: None,
    },
    // Tools
    ItemData {
        item_type: ItemType::CopperPickaxe,
        name: "Copper Pickaxe",
        icon_path: "copper_pickaxe.png",
        weight: 1.,
        block: None,
        breaking_strength: Some(100),
    },
    ItemData {
        item_type: ItemType::BronzePickaxe,
        name: "Bronze Pickaxe",
        icon_path: "bronze_pickaxe.png",
        weight: 1.,
        block: None,
        breaking_strength: Some(200),
    },
    ItemData {
        item_type: ItemType::IronPickaxe,
        name: "Iron Pickaxe",
        icon_path: "iron_pickaxe.png",
        weight: 1.,
        block: None,
        breaking_strength: Some(300),
    },
    ItemData {
        item_type: ItemType::SteelPickaxe,
        name: "Steel Pickaxe",
        icon_path: "steel_pickaxe.png",
        weight: 1.,
        block: None,
        breaking_strength: Some(400),
    },
    ItemData {
        item_type: ItemType::MagicMetalPickaxe,
        name: "Magic Metal Pickaxe",
        icon_path: "magic_metal_pickaxe.png",
        weight: 1.,
        block: None,
        breaking_strength: Some(500),
    },
];
