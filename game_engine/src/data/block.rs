use enum_map::{Enum, EnumMap};
use num_derive::{FromPrimitive, ToPrimitive};
use rand::{random_bool, random_range};

use crate::{
    data::item::ItemType,
    state::blocks::{BlockState, chest::ChestState},
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
}

#[derive(Debug, Clone)]
pub struct BlockData {
    pub(super) texture_path: &'static str,

    pub block_type: BlockType,
    /// None == unbreakable, 0 == broken by anything, bigger == harder to break
    pub hardness: Option<u32>,
    pub item_on_break: ItemType,
    pub renderable: bool,
    pub interactable: bool,
    // Default state for the block when placed
    pub state: Option<fn() -> BlockState>,
}

pub(super) const TEXTURE_FOLDER: &str = "res/meshes";
pub(super) const BLOCK_DATA: [BlockData; 13] = [
    BlockData {
        block_type: BlockType::Air,
        hardness: None,
        item_on_break: ItemType::Dirt,
        texture_path: "dirt.png",
        renderable: false,
        interactable: false,
        state: None,
    },
    BlockData {
        block_type: BlockType::Dirt,
        hardness: Some(0),
        item_on_break: ItemType::Dirt,
        texture_path: "dirt.png",
        renderable: true,
        interactable: false,
        state: None,
    },
    BlockData {
        block_type: BlockType::Stone,
        hardness: Some(10),
        item_on_break: ItemType::Stone,
        texture_path: "stone.png",
        renderable: true,
        interactable: false,
        state: None,
    },
    BlockData {
        block_type: BlockType::DarkStone,
        hardness: Some(100),
        item_on_break: ItemType::DarkStone,
        texture_path: "darkstone.png",
        renderable: true,
        interactable: false,
        state: None,
    },
    BlockData {
        block_type: BlockType::MossyStone,
        hardness: Some(10),
        item_on_break: ItemType::MossyStone,
        texture_path: "mossystone.png",
        renderable: true,
        interactable: false,
        state: None,
    },
    BlockData {
        block_type: BlockType::VoidStone,
        hardness: Some(200),
        item_on_break: ItemType::VoidStone,
        texture_path: "voidstone.png",
        renderable: true,
        interactable: false,
        state: None,
    },
    BlockData {
        block_type: BlockType::RadioactiveStone,
        hardness: Some(200),
        item_on_break: ItemType::RadioactiveStone,
        texture_path: "radioactivestone.png",
        renderable: true,
        interactable: false,
        state: None,
    },
    // Ores
    BlockData {
        block_type: BlockType::Copper,
        hardness: Some(100),
        item_on_break: ItemType::Copper,
        texture_path: "copper.png",
        renderable: true,
        interactable: false,
        state: None,
    },
    BlockData {
        block_type: BlockType::Tin,
        hardness: Some(200),
        item_on_break: ItemType::Tin,
        texture_path: "tin.png",
        renderable: true,
        interactable: false,
        state: None,
    },
    BlockData {
        block_type: BlockType::Iron,
        hardness: Some(300),
        item_on_break: ItemType::Iron,
        texture_path: "iron.png",
        renderable: true,
        interactable: false,
        state: None,
    },
    BlockData {
        block_type: BlockType::Coal,
        hardness: Some(300),
        item_on_break: ItemType::Coal,
        texture_path: "coal.png",
        renderable: true,
        interactable: false,
        state: None,
    },
    BlockData {
        block_type: BlockType::MagicMetal,
        hardness: Some(400),
        item_on_break: ItemType::MagicMetal,
        texture_path: "magic_metal.png",
        renderable: true,
        interactable: false,
        state: None,
    },
    BlockData {
        block_type: BlockType::Chest,
        hardness: Some(0),
        item_on_break: ItemType::Chest,
        texture_path: "smiley.png",
        renderable: true,
        interactable: true,
        state: Some(|| {
            BlockState::Chest(ChestState {
                items: {
                    let mut hm = EnumMap::default();

                    // Spawn with some random stuff
                    (0..ItemType::LENGTH)
                        .map(ItemType::from_usize)
                        .for_each(|item| {
                            if random_bool(0.1) {
                                hm[item] = random_range(1..=10);
                            }
                        });

                    hm
                },
            })
        }),
    },
];
