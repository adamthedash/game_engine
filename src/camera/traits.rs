use std::time::Duration;

use winit::event::KeyEvent;

use super::Camera;
use crate::world::World;

pub trait CameraController {
    fn handle_keypress(&mut self, event: &KeyEvent);
    fn handle_mouse_move(&mut self, delta: (f32, f32), camera: &mut Camera);

    fn update_camera(&mut self, camera: &mut Camera, world: &World, duration: &Duration);

    fn toggle(&mut self);
    fn enabled(&self) -> bool;
}
