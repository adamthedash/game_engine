#![feature(int_roundings)]

pub mod bbox;
pub mod block;
pub mod camera;
pub mod game;
pub mod item;
pub mod player;
pub mod render;
pub mod ui;
pub mod world;
pub mod world_gen;
pub mod util;

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
