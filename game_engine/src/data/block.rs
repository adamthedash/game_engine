use std::sync::LazyLock;

use enum_map::Enum;
use num_derive::{FromPrimitive, ToPrimitive};
use typed_builder::TypedBuilder;

use crate::{
    data::item::ItemType,
    state::{
        blocks::{BlockState, chest::ChestState, crafter::CrafterState},
        world::BlockPos,
    },
};

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
    // Interactables
    Chest,
    Crafter,
}

#[derive(TypedBuilder, Debug, Clone)]
pub struct BlockData {
    pub(super) texture_path: &'static str,
    pub block_type: BlockType,

    /// None == unbreakable, 0 == broken by anything, bigger == harder to break
    #[builder(default, setter(strip_option))]
    pub hardness: Option<u32>,

    #[builder(default, setter(strip_option))]
    pub item_on_break: Option<ItemType>,

    #[builder(default = true)]
    pub renderable: bool,

    #[builder(default)]
    pub interactable: bool,

    // Default state for the block when placed
    #[builder(default, setter(strip_option))]
    pub state: Option<fn(&BlockPos) -> BlockState>,
}

pub(super) const TEXTURE_FOLDER: &str = "res/meshes";
pub(super) static BLOCK_DATA: LazyLock<Vec<BlockData>> = LazyLock::new(|| {
    vec![
        // Basic blocks
        BlockData::builder()
            .texture_path("dirt.png")
            .block_type(BlockType::Air)
            .renderable(false)
            .build(),
        BlockData::builder()
            .texture_path("dirt.png")
            .block_type(BlockType::Dirt)
            .hardness(0)
            .item_on_break(ItemType::Dirt)
            .build(),
        BlockData::builder()
            .texture_path("stone.png")
            .block_type(BlockType::Stone)
            .hardness(10)
            .item_on_break(ItemType::Stone)
            .build(),
        BlockData::builder()
            .texture_path("darkstone.png")
            .block_type(BlockType::DarkStone)
            .hardness(100)
            .item_on_break(ItemType::DarkStone)
            .build(),
        BlockData::builder()
            .texture_path("mossystone.png")
            .block_type(BlockType::MossyStone)
            .hardness(10)
            .item_on_break(ItemType::MossyStone)
            .build(),
        BlockData::builder()
            .texture_path("voidstone.png")
            .block_type(BlockType::VoidStone)
            .hardness(200)
            .item_on_break(ItemType::VoidStone)
            .build(),
        BlockData::builder()
            .texture_path("radioactivestone.png")
            .block_type(BlockType::RadioactiveStone)
            .hardness(200)
            .item_on_break(ItemType::RadioactiveStone)
            .build(),
        // Ores
        BlockData::builder()
            .texture_path("copper.png")
            .block_type(BlockType::Copper)
            .hardness(100)
            .item_on_break(ItemType::Copper)
            .build(),
        BlockData::builder()
            .texture_path("tin.png")
            .block_type(BlockType::Tin)
            .hardness(200)
            .item_on_break(ItemType::Tin)
            .build(),
        BlockData::builder()
            .texture_path("iron.png")
            .block_type(BlockType::Iron)
            .hardness(300)
            .item_on_break(ItemType::Iron)
            .build(),
        BlockData::builder()
            .texture_path("coal.png")
            .block_type(BlockType::Coal)
            .hardness(300)
            .item_on_break(ItemType::Coal)
            .build(),
        BlockData::builder()
            .texture_path("magic_metal.png")
            .block_type(BlockType::MagicMetal)
            .hardness(400)
            .item_on_break(ItemType::MagicMetal)
            .build(),
        BlockData::builder()
            .texture_path("chest.png")
            .block_type(BlockType::Chest)
            .hardness(0)
            .item_on_break(ItemType::Chest)
            .interactable(true)
            .state(|pos| BlockState::Chest(ChestState::new(pos)))
            .build(),
        BlockData::builder()
            .texture_path("smiley.png")
            .block_type(BlockType::Crafter)
            .hardness(0)
            .item_on_break(ItemType::Crafter)
            .interactable(true)
            .state(|pos| BlockState::Crafter(CrafterState::new(pos)))
            .build(),
    ]
});
