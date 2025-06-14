#![feature(int_roundings)]
use std::{f32, path::Path, sync::Arc, time::Instant};

use camera::{Camera, walking::WalkingCameraController};
use cgmath::{Deg, Rad};
use game::GameState;
use player::Player;
use tokio::runtime::Runtime;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

use crate::{
    camera::traits::CameraController,
    render::state::RenderState,
    world::{BlockPos, BlockType, World, WorldPos},
};

mod bbox;
mod block;
mod camera;
mod game;
mod player;
mod render;
mod world;
mod world_gen;

struct App<C: CameraController> {
    runtime: Runtime,
    render_state: Option<RenderState>,
    camera_controller: C,
    game_state: GameState,
    last_update: Option<Instant>,
}

impl App<WalkingCameraController> {
    fn new() -> Self {
        let mut game_state = GameState {
            world: World::default(),
            player: Player {
                camera: Camera {
                    pos: WorldPos((-7., -20., -14.).into()),
                    yaw: Rad(0.),
                    pitch: Rad(0.),
                    aspect: 1.,
                    fovy: Deg(90.),
                    znear: 0.1,
                    zfar: 100.,
                },
                arm_length: 5.,
            },
        };
        game_state.init();

        Self {
            runtime: Runtime::new().unwrap(),
            render_state: None,
            //camera_controller: BasicFlightCameraController::new(5., 2. * f32::consts::PI * 1.),
            camera_controller: WalkingCameraController::new(
                5.,
                2. * f32::consts::PI * 0.5,
                10.,
                1.5,
            ),
            //camera_controller: SpaceFlightCameraController::new(
            //    25.,
            //    2. * f32::consts::PI * 1.,
            //    Some(5.),
            //    0.25,
            //),
            game_state,
            last_update: None,
        }
    }
}

impl<C: CameraController> ApplicationHandler for App<C> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.render_state.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
            let render_state = self.runtime.block_on(RenderState::new(window));
            render_state
                .window
                .set_cursor_grab(CursorGrabMode::Confined)
                .unwrap();
            render_state.window.set_cursor_visible(false);
            self.render_state = Some(render_state);
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        println!("Saving world...");
        self.game_state.world.save(Path::new("./saves"));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if !matches!(
            event,
            WindowEvent::RedrawRequested
                | WindowEvent::Moved { .. }
                | WindowEvent::AxisMotion { .. }
                | WindowEvent::CursorMoved { .. }
        ) {
            println!("Event: {event:?}");
        }

        // Debug block
        if let Some(block) = self
            .game_state
            .world
            .get_block_mut(&BlockPos::new(-4, 23, -5))
        {
            *block = BlockType::Smiley;
        };

        match event {
            WindowEvent::RedrawRequested => {
                // Game update pass
                if let Some(last_updated) = self.last_update {
                    let duration = Instant::now().duration_since(last_updated);
                    self.camera_controller.update_camera(
                        &mut self.game_state.player.camera,
                        &self.game_state.world,
                        &duration,
                    );
                    self.game_state.update(&duration);
                }
                self.last_update = Some(Instant::now());

                // Render pass
                if let Some(render_state) = &mut self.render_state {
                    render_state.window.request_redraw();

                    render_state.update_camera_buffer(&self.game_state.player.camera);

                    render_state.render(&self.game_state);
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(render_state) = &mut self.render_state {
                    render_state.resize(size, &mut self.game_state.player.camera);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    event @ KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        repeat,
                        state,
                        ..
                    },
                ..
            } => {
                if key == KeyCode::F4 {
                    event_loop.exit();
                }
                if key == KeyCode::Escape && !repeat && state.is_pressed() {
                    self.camera_controller.toggle();

                    // Toggle window cursor locking
                    if let Some(render_state) = &mut self.render_state {
                        if self.camera_controller.enabled() {
                            render_state.grab_cursor().expect("Failed to grab cursor");
                        } else {
                            render_state.ungrab_cursor();
                        }
                    }
                }

                self.camera_controller.handle_keypress(&event);
                self.game_state.handle_keypress(&event);
            }
            event @ WindowEvent::MouseInput { .. } => {
                self.game_state.handle_mouse_key(&event);
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(render_state) = &mut self.render_state
                    && self.camera_controller.enabled()
                {
                    let delta = (
                        position.x as f32 - (render_state.config.width / 2) as f32,
                        position.y as f32 - (render_state.config.height / 2) as f32,
                    );
                    let normalised_delta = (
                        delta.0 / render_state.config.width as f32,
                        delta.1 / render_state.config.height as f32,
                    );
                    render_state
                        .centre_cursor()
                        .expect("Failed to centre cursor");

                    self.camera_controller
                        .handle_mouse_move(normalised_delta, &mut self.game_state.player.camera);
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
