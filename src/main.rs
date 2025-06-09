use crate::render::RenderState;
use std::sync::Arc;

use camera::{Camera, CameraController};
use tokio::runtime::Runtime;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

mod block;
mod camera;
mod chunk;
mod render;
mod texture;

struct App<'a> {
    runtime: Runtime,
    render_state: Option<RenderState<'a>>,
    camera_controller: CameraController,
}

impl App<'_> {
    fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            render_state: None,
            camera_controller: CameraController::new(0.2, 0.2),
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
                pos: (0., 1., 2.).into(),
                forward: cgmath::Vector3::unit_x(),
                up: cgmath::Vector3::unit_y(),
                aspect: 1.,
                fovy: 45.,
                znear: 0.1,
                zfar: 100.,
            };
            let render_state = self.runtime.block_on(RenderState::new(window, camera));
            render_state
                .window
                .set_cursor_grab(CursorGrabMode::Confined)
                .unwrap();
            self.render_state = Some(render_state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        println!("Event: {:?}", event);

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

                    render_state.render();
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(render_state) = &mut self.render_state {
                    render_state.resize(size);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(key),
                        ..
                    },
                ..
            } => {
                if key == KeyCode::F4 {
                    println!("Closing");
                    event_loop.exit();
                }

                self.camera_controller.handle_keypress(key, state);
            }
            WindowEvent::AxisMotion { axis, value, .. } => {}
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
