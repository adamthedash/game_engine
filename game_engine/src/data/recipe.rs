use std::sync::LazyLock;

use rustc_hash::FxHashMap;

use crate::data::item::ItemType;

#[derive(Debug)]
pub struct Recipe {
    pub inputs: FxHashMap<ItemType, usize>,
    pub output: (ItemType, usize),
}

pub static RECIPES: LazyLock<Vec<Recipe>> = LazyLock::new(|| {
    vec![
        Recipe {
            inputs: { FxHashMap::from_iter([(ItemType::Stone, 4)]) },
            output: (ItemType::DarkStone, 1),
        },
        Recipe {
            inputs: { FxHashMap::from_iter([(ItemType::Dirt, 1), (ItemType::Stone, 2)]) },
            output: (ItemType::MossyStone, 2),
        },
    ]
});
