use std::time::Duration;

use winit::event::KeyEvent;

use crate::state::{
    player::{Player, Position},
    world::World,
};

pub trait PlayerController {
    fn handle_keypress(&mut self, event: &KeyEvent);

    /// Handle mouse movement. Return whether the player's position has changed
    fn handle_mouse_move(&mut self, delta: (f32, f32), player: &mut Position) -> bool;

    /// Move the player. Return whether the player's position has changed
    fn move_player(&mut self, player: &mut Player, world: &World, duration: &Duration) -> bool;

    fn toggle(&mut self);
    fn enabled(&self) -> bool;
}
