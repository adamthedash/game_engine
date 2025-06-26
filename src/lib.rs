#![feature(int_roundings)]
#![feature(iter_collect_into)]
#![feature(array_windows)]

pub mod bbox;
pub mod block;
pub mod camera;
pub mod data;
pub mod game;
pub mod perlin_cdf;
pub mod player;
pub mod render;
pub mod ui;
pub mod util;
pub mod world;
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
