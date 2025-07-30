use std::sync::LazyLock;

use enum_map::Enum;
use typed_builder::TypedBuilder;

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
    // Interactable blocks
    Chest,
}

// User-defined
#[derive(Clone, Debug, TypedBuilder)]
pub struct ItemData {
    pub(super) icon_path: &'static str,
    pub item_type: ItemType,
    pub name: &'static str,

    #[builder(default)]
    pub weight: f32,

    #[builder(default, setter(strip_option))]
    pub block: Option<BlockType>,

    #[builder(default, setter(strip_option))]
    pub breaking_strength: Option<u32>,
}

pub(super) const ICON_PATH: &str = "res/icons";
pub(super) static ITEM_DATA: LazyLock<Vec<ItemData>> = LazyLock::new(|| {
    vec![
        ItemData::builder()
            .item_type(ItemType::Dirt)
            .name("Dirt")
            .icon_path("dirt.png")
            .weight(1.)
            .block(BlockType::Dirt)
            .build(),
        ItemData::builder()
            .item_type(ItemType::Stone)
            .name("Stone")
            .icon_path("stone.png")
            .weight(1.)
            .block(BlockType::Stone)
            .build(),
        ItemData::builder()
            .item_type(ItemType::DarkStone)
            .name("Dark Stone")
            .icon_path("darkstone.png")
            .weight(1.)
            .block(BlockType::DarkStone)
            .build(),
        ItemData::builder()
            .item_type(ItemType::MossyStone)
            .name("Mossy Stone")
            .icon_path("mossystone.png")
            .weight(1.)
            .block(BlockType::MossyStone)
            .build(),
        ItemData::builder()
            .item_type(ItemType::VoidStone)
            .name("Void Stone")
            .icon_path("voidstone.png")
            .weight(1.)
            .block(BlockType::VoidStone)
            .build(),
        ItemData::builder()
            .item_type(ItemType::RadioactiveStone)
            .name("Radioactive Stone")
            .icon_path("radioactivestone.png")
            .weight(1.)
            .block(BlockType::RadioactiveStone)
            .build(),
        // Ores
        ItemData::builder()
            .item_type(ItemType::Copper)
            .name("Copper Ore")
            .icon_path("copper.png")
            .weight(1.)
            .block(BlockType::Copper)
            .build(),
        ItemData::builder()
            .item_type(ItemType::Tin)
            .name("Tin Ore")
            .icon_path("tin.png")
            .weight(1.)
            .block(BlockType::Tin)
            .build(),
        ItemData::builder()
            .item_type(ItemType::Bronze)
            .name("Bronze Bar")
            .icon_path("bronze.png")
            .weight(1.)
            .build(),
        ItemData::builder()
            .item_type(ItemType::Coal)
            .name("Coal")
            .icon_path("coal.png")
            .weight(1.)
            .block(BlockType::Coal)
            .build(),
        ItemData::builder()
            .item_type(ItemType::Iron)
            .name("Iron Ore")
            .icon_path("iron.png")
            .weight(1.)
            .block(BlockType::Iron)
            .build(),
        ItemData::builder()
            .item_type(ItemType::Steel)
            .name("Steel Bar")
            .icon_path("steel.png")
            .weight(1.)
            .build(),
        ItemData::builder()
            .item_type(ItemType::MagicMetal)
            .name("Magic Metal Bar")
            .icon_path("magic_metal.png")
            .weight(1.)
            .block(BlockType::MagicMetal)
            .build(),
        // Tools
        ItemData::builder()
            .item_type(ItemType::CopperPickaxe)
            .name("Copper Pickaxe")
            .icon_path("copper_pickaxe.png")
            .weight(1.)
            .breaking_strength(100)
            .build(),
        ItemData::builder()
            .item_type(ItemType::BronzePickaxe)
            .name("Bronze Pickaxe")
            .icon_path("bronze_pickaxe.png")
            .weight(1.)
            .breaking_strength(200)
            .build(),
        ItemData::builder()
            .item_type(ItemType::IronPickaxe)
            .name("Iron Pickaxe")
            .icon_path("iron_pickaxe.png")
            .weight(1.)
            .breaking_strength(300)
            .build(),
        ItemData::builder()
            .item_type(ItemType::SteelPickaxe)
            .name("Steel Pickaxe")
            .icon_path("steel_pickaxe.png")
            .weight(1.)
            .breaking_strength(400)
            .build(),
        ItemData::builder()
            .item_type(ItemType::MagicMetalPickaxe)
            .name("Magic Metal Pickaxe")
            .icon_path("magic_metal_pickaxe.png")
            .weight(1.)
            .breaking_strength(500)
            .build(),
        ItemData::builder()
            .item_type(ItemType::Chest)
            .name("Chest")
            .icon_path("chest.png")
            .weight(1.)
            .block(BlockType::Chest)
            .build(),
    ]
});
