use hecs::Entity;
use rustc_hash::FxHashMap;

use crate::{
    entity::components::{Container, Crafter, UprightOrientation},
    state::world::{BlockPos, WorldPos},
};

pub type CrafterBlock = (BlockPos, Container, Crafter);
pub type Chest = (BlockPos, Container);
pub type Player = (WorldPos, UprightOrientation, Container);
pub type Monster = (WorldPos, UprightOrientation);

pub type BlockStates = FxHashMap<BlockPos, Entity>;
