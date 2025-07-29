use std::time::Duration;

use winit::event::KeyEvent;

use crate::state::{
    player::{Player, Position},
    world::World,
};

pub trait PlayerController {
    fn handle_keypress(&mut self, event: &KeyEvent);
    fn handle_mouse_move(&mut self, delta: (f32, f32), player: &mut Position);

    fn move_player(&mut self, player: &mut Player, world: &World, duration: &Duration);

    fn toggle(&mut self);
    fn enabled(&self) -> bool;
}
