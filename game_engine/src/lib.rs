#![feature(int_roundings)]
#![feature(iter_collect_into)]
#![feature(array_windows)]
#![feature(array_repeat)]
#![feature(slice_split_once)]

#[macro_use]
extern crate impl_ops;

use crate::state::world::BlockPos;

pub mod block;
pub mod camera;
pub mod data;
pub mod event;
pub mod math;
pub mod perlin_cdf;
pub mod render;
pub mod state;
pub mod ui;
pub mod util;
pub mod world_gen;
pub mod entity;

#[derive(PartialEq, Debug, Clone)]
pub enum InteractionMode {
    // Player can walk around and interact with the world
    Game,
    // Player is in the personal interface (inventory, etc.)
    UI,
    // Player is in an interface in the world
    Block(BlockPos),
}
