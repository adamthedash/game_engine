#![feature(int_roundings)]
use std::{sync::Arc, time::Instant};

use game_engine::{
    InteractionMode,
    camera::{Controller, traits::PlayerController},
    entity::{components::Hotbar, systems::spawn_player},
    event::{
        MESSAGE_QUEUE, Message, Subscriber,
        messages::{
            ItemFavouritedMessage, TransferItemMessage, TransferItemRequestMessage,
            TransferItemSource,
        },
    },
    render::state::RenderState,
    state::{game::GameState, world::World},
    ui::debug::DEBUG_WINDOW,
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
}

impl App {
    fn new() -> Self {
        let mut ecs = hecs::World::new();
        let player_entity = spawn_player(&mut ecs);

        let mut game_state = GameState {
            world: World::default(),
            player: player_entity,
            entities: vec![],
            ecs,
        };
        game_state.init();

        Self {
            runtime: Runtime::new().unwrap(),
            render_state: None,
            player_controller: Controller::default_walking(),
            game_state,
            last_update: None,
        }
    }
}

impl App {
    /// Process all the messages in the queue, routing them to their subscribers
    pub fn process_message_queue(&mut self) {
        use Message::*;
        while let Some(m) = MESSAGE_QUEUE.take() {
            log::debug!("Message: {m:?}");
            match &m {
                ToggleInteractionMode => {
                    let interaction_mode = self
                        .game_state
                        .ecs
                        .get::<&InteractionMode>(self.game_state.player)
                        .unwrap();

                    use InteractionMode::*;
                    MESSAGE_QUEUE.send(SetInteractionMode(match *interaction_mode {
                        Game => UI,
                        UI | Block(_) => Game,
                    }));
                }
                SetInteractionMode(mode) => {
                    let mut interaction_mode = self
                        .game_state
                        .ecs
                        .get::<&mut InteractionMode>(self.game_state.player)
                        .unwrap();
                    *interaction_mode = mode.clone();
                }
                // TODO: This way of doing transfers is ugly af, see if we can find a cleaner way.
                TransferItemRequest(TransferItemRequestMessage {
                    item,
                    count,
                    source,
                }) => {
                    let interaction_mode = &*self
                        .game_state
                        .ecs
                        .get::<&InteractionMode>(self.game_state.player)
                        .unwrap();

                    if let Some((source, dest)) = match (source, interaction_mode) {
                        // Player -> Block
                        (TransferItemSource::Inventory, InteractionMode::Block(target_block)) => {
                            let dest = *self
                                .game_state
                                .world
                                .block_states
                                .get(target_block)
                                .expect("Tried to transfer to non-stateful block!");

                            Some((self.game_state.player, dest))
                        }
                        // Block -> Player
                        (
                            TransferItemSource::Block(source_block),
                            InteractionMode::Block(target_block),
                        ) if source_block == target_block => {
                            let source = *self
                                .game_state
                                .world
                                .block_states
                                .get(source_block)
                                .expect("Tried to transfer from non-stateful block!");

                            Some((source, self.game_state.player))
                        }
                        // TODO: Block -> Block?
                        _ => {
                            // Nothing
                            None
                        }
                    } {
                        MESSAGE_QUEUE.send(TransferItem(TransferItemMessage {
                            source,
                            dest,
                            item: *item,
                            count: *count,
                        }));
                    }
                }
                &Message::ItemFavourited(ItemFavouritedMessage { item, slot }) => {
                    let mut hotbar = self
                        .game_state
                        .ecs
                        .get::<&mut Hotbar>(self.game_state.player)
                        .unwrap();
                    hotbar.set_favourite(slot, item);
                }
                _ => (),
            }

            // Handle game state events first
            self.game_state.handle_message(&m);
            self.game_state.world.handle_message(&m);

            // Then player interaction events
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
            && self.player_controller.enabled()
        {
            let is_game_mode = {
                let interaction_mode = self
                    .game_state
                    .ecs
                    .get::<&InteractionMode>(self.game_state.player)
                    .unwrap();
                matches!(*interaction_mode, InteractionMode::Game)
            };

            if is_game_mode {
                let config = &render_state.draw_context.config;
                let normalised_delta = (
                    delta.0 as f32 / config.width as f32,
                    delta.1 as f32 / config.height as f32,
                );
                if render_state.draw_context.centre_cursor().is_err() {
                    log::warn!("WARNING: Failed to centre cursor!");
                }

                self.player_controller.handle_mouse_move(
                    &mut self.game_state.ecs,
                    self.game_state.player,
                    normalised_delta,
                );
            }
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
                    self.player_controller.move_entity(
                        &mut self.game_state.ecs,
                        self.game_state.player,
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

                    render_state.render(&self.game_state);
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
                    MESSAGE_QUEUE.send(Message::ToggleInteractionMode);
                }

                self.player_controller.handle_keypress(event);
                self.game_state.handle_keypress(event);
            }
            event @ WindowEvent::MouseInput { .. } => {
                self.game_state.handle_mouse_key(event);
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                let interaction_mode = self
                    .game_state
                    .ecs
                    .get::<&InteractionMode>(self.game_state.player)
                    .unwrap();
                if matches!(*interaction_mode, InteractionMode::Game) && *y != 0. {
                    let mut hotbar = self
                        .game_state
                        .ecs
                        .get::<&mut Hotbar>(self.game_state.player)
                        .unwrap();
                    hotbar.scroll(*y < 0.);
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
