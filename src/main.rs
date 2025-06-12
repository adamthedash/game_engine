#![feature(int_roundings)]
use std::{f32, path::Path, sync::Arc};

use camera::{Camera, CameraController};
use cgmath::{Deg, Rad};
use tokio::runtime::Runtime;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{chunk::World, render::RenderState};

mod block;
mod camera;
mod chunk;
mod model;
mod render;
mod shader;
mod texture;
mod world_gen;

struct App<'a> {
    runtime: Runtime,
    render_state: Option<RenderState<'a>>,
    camera_controller: CameraController,
    prev_cursor_pos: (Option<f32>, Option<f32>),
    world: World,
}

impl App<'_> {
    fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            render_state: None,
            camera_controller: CameraController::new(0.2, 2. * f32::consts::PI * 1.),
            prev_cursor_pos: (None, None),
            world: World::default(),
        }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.render_state.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
            // Initial position of the camera/player
            let camera = Camera {
                pos: (-10., 10., -10.).into(),
                yaw: Rad(0.),
                pitch: Rad(0.),
                aspect: 1.,
                fovy: Deg(45.),
                znear: 0.1,
                zfar: 100.,
            };
            let render_state = self.runtime.block_on(RenderState::new(window, camera));
            // render_state
            //     .window
            //     .set_cursor_grab(CursorGrabMode::Confined)
            //     .unwrap();
            self.render_state = Some(render_state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if !matches!(event, WindowEvent::RedrawRequested) {
            println!("Event: {:?}", event);
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("Closing");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(render_state) = &mut self.render_state {
                    render_state.window.request_redraw();

                    self.camera_controller
                        .update_camera(&mut render_state.camera);
                    render_state.update_camera_buffer();

                    render_state.render(&mut self.world);
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(render_state) = &mut self.render_state {
                    render_state.resize(size);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    event @ KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        ..
                    },
                ..
            } => {
                if key == KeyCode::F4 {
                    println!("Closing");
                    self.world.save(Path::new("./saves"));
                    event_loop.exit();
                }
                self.camera_controller.handle_keypress(&event);
            }
            WindowEvent::AxisMotion { axis, value, .. } => {
                if let Some(render_state) = &mut self.render_state {
                    let value = value as f32;
                    let normalised = match axis {
                        0 => {
                            let cur = value / render_state.config.width as f32;
                            let prev = self.prev_cursor_pos.0.unwrap_or(cur);
                            self.prev_cursor_pos.0 = Some(cur);
                            cur - prev
                        }
                        1 => {
                            let cur = value / render_state.config.height as f32;
                            let prev = self.prev_cursor_pos.1.unwrap_or(cur);
                            self.prev_cursor_pos.1 = Some(cur);
                            cur - prev
                        }
                        _ => panic!("Unknown axis"),
                    };

                    self.camera_controller.handle_mouse_move(
                        axis,
                        normalised,
                        &mut render_state.camera,
                    );
                }
            }
            _ => {}
        }
    }
}

fn main() {
    /*

    Following https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface
    Adapted to v30 wgpu based on https://github.com/gfx-rs/wgpu/blob/trunk/examples/standalone/02_hello_window/src/main.rs

    */
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
