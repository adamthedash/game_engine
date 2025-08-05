#![feature(int_roundings)]
use std::{cell::RefCell, f32, rc::Rc, sync::Arc, time::Instant};

use cgmath::Rad;
use enum_map::EnumMap;
use game_engine::{
    InteractionMode,
    camera::{Controller, traits::PlayerController},
    data::item::ItemType,
    event::{MESSAGE_QUEUE, Message, Subscriber},
    render::state::RenderState,
    state::{
        game::{GameState, TransferItemMessage},
        player::{Player, Position},
        world::{World, WorldPos},
    },
    ui::{
        debug::DEBUG_WINDOW,
        hotbar::Hotbar,
        inventory::{Inventory, TransferItemRequestMessage, TransferItemSource},
    },
    util::stopwatch::StopWatch,
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

struct App {
    runtime: Runtime,
    render_state: Option<RenderState>,
    player_controller: Controller,
    game_state: GameState,
    last_update: Option<Instant>,
    interaction_mode: InteractionMode,
}

impl App {
    fn new() -> Self {
        let inventory = Inventory {
            items: {
                let mut items = EnumMap::default();
                items[ItemType::Dirt] = 5;
                items[ItemType::Stone] = 12;
                items[ItemType::Coal] = 12;
                items[ItemType::Iron] = 12;
                items[ItemType::Copper] = 12;
                items[ItemType::Tin] = 12;
                items[ItemType::Bronze] = 12;
                items[ItemType::Steel] = 12;
                items[ItemType::MagicMetal] = 12;
                items[ItemType::Chest] = 12;

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
            world: World::default(),
            player: Player {
                position: Position {
                    pos: WorldPos((-7., -20., -14.).into()),
                    yaw: Rad(0.),
                    pitch: Rad(0.),
                },
                vision_distance: 100.,
                arm_length: 5.,
                hotbar,
                inventory,
            },
        };
        game_state.init();

        Self {
            runtime: Runtime::new().unwrap(),
            render_state: None,
            player_controller: Controller::default_walking(),
            game_state,
            last_update: None,
            interaction_mode: InteractionMode::Game,
        }
    }
}

impl App {
    /// Process all the messages in the queue, routing them to their subscribers
    pub fn process_message_queue(&mut self) {
        use Message::*;
        while let Some(m) = MESSAGE_QUEUE.take() {
            log::debug!("Messag: {m:?}");
            match &m {
                SetInteractionMode(mode) => {
                    self.interaction_mode = mode.clone();
                }
                // TODO: This way of doing transfers is ugly af, see if we can find a cleaner way.
                TransferItemRequest(TransferItemRequestMessage {
                    item,
                    count,
                    source,
                }) => {
                    // Inventory -> Block
                    if let TransferItemSource::Inventory = source
                        && let InteractionMode::Block(pos) = &self.interaction_mode
                    {
                        MESSAGE_QUEUE.send(TransferItem(TransferItemMessage {
                            source: source.clone(),
                            dest: TransferItemSource::Block(pos.clone()),
                            item: *item,
                            count: *count,
                        }));

                    // Block -> Inventory
                    } else if matches!(source, TransferItemSource::Block(..)) {
                        MESSAGE_QUEUE.send(TransferItem(TransferItemMessage {
                            source: source.clone(),
                            dest: TransferItemSource::Inventory,
                            item: *item,
                            count: *count,
                        }));
                    }
                    // TODO: Block -> Block?
                }
                _ => (),
            }

            // Handle game state events first
            self.game_state.handle_message(&m);
            self.game_state.world.handle_message(&m);

            // Then player interaction events
            self.game_state.player.hotbar.handle_message(&m);
            self.player_controller.handle_message(&m);

            // Then rendering events
            if let Some(render_state) = &mut self.render_state {
                render_state.handle_message(&m);
            }
        }
    }
}

impl ApplicationHandler for App {
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
            let render_state = self
                .runtime
                .block_on(RenderState::new(window, &self.game_state.player.position));

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
        log::info!("Saving world...");
        //self.game_state.world.save(Path::new("./saves"));
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event
            && let Some(render_state) = &mut self.render_state
            && matches!(self.interaction_mode, InteractionMode::Game)
            && self.player_controller.enabled()
        {
            let config = &render_state.draw_context.config;
            let normalised_delta = (
                delta.0 as f32 / config.width as f32,
                delta.1 as f32 / config.height as f32,
            );
            if render_state.draw_context.centre_cursor().is_err() {
                log::warn!("WARNING: Failed to centre cursor!");
            }

            self.player_controller
                .handle_mouse_move(normalised_delta, &mut self.game_state.player.position);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        log::trace!("Event: {event:?}");

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
                // Game update pass
                // TODO: separate game loop and render loop
                if let Some(last_updated) = self.last_update {
                    let duration = Instant::now().duration_since(last_updated);
                    DEBUG_WINDOW.add_line(&format!("Last frame: {duration:?}"));
                    self.player_controller.move_player(
                        &mut self.game_state.player,
                        &self.game_state.world,
                        &duration,
                    );
                    self.game_state.tick(&duration);
                }
                self.last_update = Some(Instant::now());

                // Process message queue
                let mut stopwatch = StopWatch::new();
                self.process_message_queue();
                stopwatch.stamp_and_reset("Message proccesing");
                stopwatch
                    .get_debug_strings()
                    .iter()
                    .for_each(|l| DEBUG_WINDOW.add_line(l));

                // Render pass
                if let Some(render_state) = &mut self.render_state {
                    render_state.draw_context.window.request_redraw();

                    render_state.update_camera_buffer();

                    render_state.render(&self.game_state, &self.interaction_mode);
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(render_state) = &mut self.render_state {
                    render_state.resize(*size);
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
                    // Close interface or open inventory
                    MESSAGE_QUEUE.send(Message::SetInteractionMode({
                        use InteractionMode::*;
                        match self.interaction_mode {
                            Game => UI,
                            UI | Block(_) => Game,
                        }
                    }));
                }

                self.player_controller.handle_keypress(event);
                self.game_state.handle_keypress(event);
            }
            event @ WindowEvent::MouseInput { .. } => {
                self.game_state
                    .handle_mouse_key(event, &mut self.interaction_mode);
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                if matches!(self.interaction_mode, InteractionMode::Game) && *y != 0. {
                    self.game_state.player.hotbar.scroll(*y < 0.);
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
