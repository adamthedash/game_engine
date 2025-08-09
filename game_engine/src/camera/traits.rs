use std::time::Duration;

use hecs::Entity;
use winit::event::KeyEvent;

use crate::state::world::World;

pub trait PlayerController {
    fn handle_keypress(&mut self, event: &KeyEvent);

    /// Handle mouse movement. Return whether the player's position has changed
    fn handle_mouse_move(
        &mut self,
        ecs: &mut hecs::World,
        entity: Entity,
        delta: (f32, f32),
    ) -> bool;

    /// Move the player. Return whether the player's position has changed
    fn move_entity(
        &mut self,
        ecs: &mut hecs::World,
        entity: Entity,
        world: &World,
        duration: &Duration,
    ) -> bool;

    fn toggle(&mut self);
    fn enabled(&self) -> bool;
}
