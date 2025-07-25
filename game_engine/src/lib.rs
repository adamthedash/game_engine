#![feature(int_roundings)]
#![feature(iter_collect_into)]
#![feature(array_windows)]
#![feature(array_repeat)]

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

#[derive(PartialEq)]
pub enum InteractionMode {
    // Player can walk around and interact with the world
    Game,
    // Player is in a menu / interface
    UI,
}

impl InteractionMode {
    pub fn toggle(&mut self) {
        use InteractionMode::*;
        *self = match self {
            Game => UI,
            UI => Game,
        }
    }
}
