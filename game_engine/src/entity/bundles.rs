/*
Bundles of types that live in the ECS. Mostly for self-documentation
*/

use hecs::Entity;
use rustc_hash::FxHashMap;

use crate::{
    InteractionMode,
    entity::components::{
        Behaviour, Container, Crafter, Hotbar, Movement, Reach, UIType, UprightOrientation, Vision,
    },
    math::bbox::AABB,
    state::world::{BlockPos, WorldPos},
};

// Block Types
pub type CrafterBlock = (BlockPos, UIType, Container, Crafter);
pub type Chest = (BlockPos, UIType, Container);

pub type Player = (
    WorldPos,
    UprightOrientation,
    Container,
    Hotbar,
    Vision,
    Reach,
    // Model coordinates AABB
    AABB<f32>,
    InteractionMode,
);
pub type Monster = (
    WorldPos,
    UprightOrientation,
    Behaviour,
    AABB<f32>,
    Movement,
    Vision,
);

pub type BlockStates = FxHashMap<BlockPos, Entity>;
