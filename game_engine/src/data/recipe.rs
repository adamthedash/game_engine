use std::sync::LazyLock;

use rustc_hash::FxHashMap;

use crate::data::item::ItemType;

#[derive(Debug, Clone)]
pub struct Recipe {
    pub inputs: FxHashMap<ItemType, usize>,
    pub output: (ItemType, usize),
    pub crafting_juice_cost: f32,
}

pub static RECIPES: LazyLock<Vec<Recipe>> = LazyLock::new(|| {
    vec![
        Recipe {
            inputs: { FxHashMap::from_iter([(ItemType::Stone, 4)]) },
            output: (ItemType::DarkStone, 1),
            crafting_juice_cost: 10.,
        },
        Recipe {
            inputs: { FxHashMap::from_iter([(ItemType::Dirt, 1), (ItemType::Stone, 2)]) },
            output: (ItemType::MossyStone, 2),
            crafting_juice_cost: 10.,
        },
        Recipe {
            inputs: { FxHashMap::from_iter([(ItemType::Copper, 1), (ItemType::Tin, 1)]) },
            output: (ItemType::Bronze, 2),
            crafting_juice_cost: 10.,
        },
        Recipe {
            inputs: { FxHashMap::from_iter([(ItemType::Iron, 1), (ItemType::Coal, 1)]) },
            output: (ItemType::Steel, 1),
            crafting_juice_cost: 10.,
        },
        Recipe {
            inputs: { FxHashMap::from_iter([(ItemType::Copper, 5)]) },
            output: (ItemType::CopperPickaxe, 1),
            crafting_juice_cost: 10.,
        },
        Recipe {
            inputs: { FxHashMap::from_iter([(ItemType::Bronze, 5)]) },
            output: (ItemType::BronzePickaxe, 1),
            crafting_juice_cost: 10.,
        },
        Recipe {
            inputs: { FxHashMap::from_iter([(ItemType::Iron, 5)]) },
            output: (ItemType::IronPickaxe, 1),
            crafting_juice_cost: 10.,
        },
        Recipe {
            inputs: { FxHashMap::from_iter([(ItemType::Steel, 5)]) },
            output: (ItemType::SteelPickaxe, 1),
            crafting_juice_cost: 10.,
        },
        Recipe {
            inputs: { FxHashMap::from_iter([(ItemType::MagicMetal, 5)]) },
            output: (ItemType::MagicMetalPickaxe, 1),
            crafting_juice_cost: 10.,
        },
    ]
});
