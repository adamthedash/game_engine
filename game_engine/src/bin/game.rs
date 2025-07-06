#![feature(int_roundings)]
use std::{cell::RefCell, f32, path::Path, rc::Rc, sync::Arc, time::Instant};

use cgmath::{Deg, Rad};
use enum_map::EnumMap;
use game_engine::{
    InteractionMode,
    camera::{Camera, basic_flight::BasicFlightCameraController, traits::CameraController},
    data::{item::ItemType, world_gen::DefaultGenerator},
    event::{MESSAGE_QUEUE, Message, Subscriber},
    game::GameState,
    player::Player,
    render::state::RenderState,
    ui::{hotbar::Hotbar, inventory::Inventory},
    world::{World, WorldPos},
    world_gen::ChunkGenerator,
};
use tokio::runtime::Runtime;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

struct App<C: CameraController, G: ChunkGenerator> {
    runtime: Runtime,
    render_state: Option<RenderState>,
    camera_controller: C,
    game_state: GameState<G>,
    last_update: Option<Instant>,
    interaction_mode: InteractionMode,
}

impl App<BasicFlightCameraController, DefaultGenerator> {
    fn new() -> Self {
        let inventory = Inventory {
            items: {
                let mut items = EnumMap::default();
                items[ItemType::Dirt] = 5;
                items[ItemType::Stone] = 12;

                items
            },
        };
        let inventory = Rc::new(RefCell::new(inventory));
        let hotbar = Hotbar {
            slots: {
                let mut slots = [None; 10];
                slots[4] = Some(ItemType::Dirt);
                slots[2] = Some(ItemType::Stone);

                slots
            },
            inventory: inventory.clone(),
            selected: 0,
        };

        let mut game_state = GameState {
            world: World::<DefaultGenerator>::default(),
            player: Player {
                camera: Camera::new(
                    WorldPos((-7., -20., -14.).into()),
                    Rad(0.),
                    Rad(0.),
                    1.,
                    Deg(90.),
                    0.1,
                    100.,
                ),
                arm_length: 5.,
                hotbar,
                inventory,
            },
        };
        game_state.init();

        Self {
            runtime: Runtime::new().unwrap(),
            render_state: None,
            camera_controller: BasicFlightCameraController::new(5., 2. * f32::consts::PI * 1.),
            //camera_controller: WalkingCameraController::new(
            //    5.,
            //    2. * f32::consts::PI * 0.5,
            //    10.,
            //    1.5,
            //),
            //camera_controller: SpaceFlightCameraController::new(
            //    25.,
            //    2. * f32::consts::PI * 1.,
            //    Some(5.),
            //    0.25,
            //),
            game_state,
            last_update: None,
            interaction_mode: InteractionMode::Game,
        }
    }
}

impl<C: CameraController, G: ChunkGenerator> App<C, G> {
    /// Process all the messages in the queue, routing them to their subscribers
    pub fn process_message_queue(&mut self) {
        use Message::*;
        MESSAGE_QUEUE
            .lock()
            .unwrap()
            .drain(..)
            .for_each(|m| match &m {
                ItemFavourited(_) => {
                    self.game_state.player.hotbar.handle_message(&m);
                }
                BlockChanged(_) => self.game_state.world.handle_message(&m),
                _ => (),
            });
    }
}

impl<C: CameraController, G: ChunkGenerator> ApplicationHandler for App<C, G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Initialise RenderState once
        if self.render_state.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes().with_inner_size(PhysicalSize::new(1600, 900)),
                    )
                    .unwrap(),
            );
            let render_state = self.runtime.block_on(RenderState::new(window));

            // Lock cursor
            render_state
                .draw_context
                .window
                .set_cursor_grab(CursorGrabMode::Confined)
                .unwrap();
            render_state.draw_context.window.set_cursor_visible(false);

            self.render_state = Some(render_state);
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
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

        // Handle UI events
        if let Some(render_state) = &mut self.render_state {
            let _ = render_state
                .ui
                .egui_state
                .on_window_event(&render_state.draw_context.window, &event);
        }

        match &event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                // Process message queue
                self.process_message_queue();

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
                    render_state.draw_context.window.request_redraw();

                    render_state.update_camera_buffer(&self.game_state.player.camera);

                    render_state.render(&self.game_state, &self.interaction_mode);
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(render_state) = &mut self.render_state {
                    render_state.resize(*size, &mut self.game_state.player.camera);
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
                if *key == KeyCode::F4 {
                    event_loop.exit();
                }

                if *key == KeyCode::Escape && !repeat && state.is_pressed() {
                    self.camera_controller.toggle();
                    self.interaction_mode.toggle();

                    // Toggle window cursor locking
                    if let Some(render_state) = &mut self.render_state {
                        match self.interaction_mode {
                            InteractionMode::Game => {
                                if render_state.draw_context.grab_cursor().is_err() {
                                    println!("WARNING: Failed to grab cursor!");
                                }
                            }
                            InteractionMode::UI => {
                                render_state.draw_context.ungrab_cursor();
                            }
                        }
                    }
                }

                self.camera_controller.handle_keypress(event);
                self.game_state.handle_keypress(event);
            }
            event @ WindowEvent::MouseInput { .. } => {
                self.game_state
                    .handle_mouse_key(event, &self.interaction_mode);
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                if self.interaction_mode == InteractionMode::Game && *y != 0. {
                    self.game_state.player.hotbar.scroll(*y > 0.);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(render_state) = &mut self.render_state
                    && self.interaction_mode == InteractionMode::Game
                    && self.camera_controller.enabled()
                {
                    let config = &render_state.draw_context.config;
                    let delta = (
                        position.x as f32 - (config.width / 2) as f32,
                        position.y as f32 - (config.height / 2) as f32,
                    );
                    let normalised_delta = (
                        delta.0 / config.width as f32,
                        delta.1 / config.height as f32,
                    );
                    if render_state.draw_context.centre_cursor().is_err() {
                        println!("WARNING: Failed to centre cursor!");
                    }

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

    let _ = sycamore_reactive::create_root(|| {
        let mut app = App::new();
        event_loop.run_app(&mut app).unwrap();
    });
}
