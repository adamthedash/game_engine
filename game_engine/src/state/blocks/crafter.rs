use super::StatefulBlock;
use crate::{data::recipe::Recipe, ui::Drawable};

#[derive(Debug, Clone)]
pub struct CrafterState {
    recipe: Recipe,
}

impl StatefulBlock for CrafterState {}

impl Drawable for CrafterState {}
