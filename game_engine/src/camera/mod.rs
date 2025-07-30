use std::{f32::consts::PI, time::Duration};

use winit::event::KeyEvent;

use crate::{
    InteractionMode,
    camera::{
        basic_flight::BasicFlightController, space_flight::SpaceFlightController,
        traits::PlayerController, walking::WalkingController,
    },
    event::{MESSAGE_QUEUE, Message, Subscriber},
    state::{
        player::{Player, Position},
        world::World,
    },
};

pub mod basic_flight;
pub mod collision;
pub mod space_flight;
pub mod traits;
pub mod walking;

pub enum Controller {
    BasicFlight(BasicFlightController),
    SpaceFlight(SpaceFlightController),
    Walking(WalkingController),
}

impl Controller {
    fn as_controller_mut(&mut self) -> &mut dyn PlayerController {
        use Controller::*;
        match self {
            BasicFlight(controller) => controller,
            SpaceFlight(controller) => controller,
            Walking(controller) => controller,
        }
    }
    fn as_controller(&self) -> &dyn PlayerController {
        use Controller::*;
        match self {
            BasicFlight(controller) => controller,
            SpaceFlight(controller) => controller,
            Walking(controller) => controller,
        }
    }

    pub fn default_walking() -> Self {
        Self::Walking(WalkingController::new(5., 2. * PI * 0.5, 10., 1.5))
    }

    pub fn default_space_flight() -> Self {
        Self::SpaceFlight(SpaceFlightController::new(
            25.,
            2. * PI * 1.,
            Some(5.),
            0.25,
        ))
    }

    pub fn default_basic_flight() -> Self {
        Self::BasicFlight(BasicFlightController::new(5., 2. * PI * 1.))
    }
}

/// Pass-through implementation
impl PlayerController for Controller {
    fn handle_keypress(&mut self, event: &KeyEvent) {
        self.as_controller_mut().handle_keypress(event);
    }

    fn handle_mouse_move(&mut self, delta: (f32, f32), player_position: &mut Position) {
        self.as_controller_mut()
            .handle_mouse_move(delta, player_position);
        MESSAGE_QUEUE.send(Message::PlayerMoved(player_position.clone()));
    }

    fn move_player(&mut self, player: &mut Player, world: &World, duration: &Duration) {
        self.as_controller_mut()
            .move_player(player, world, duration);
        MESSAGE_QUEUE.send(Message::PlayerMoved(player.position.clone()));
    }

    fn toggle(&mut self) {
        self.as_controller_mut().toggle();
    }

    fn enabled(&self) -> bool {
        self.as_controller().enabled()
    }
}

impl Subscriber for Controller {
    fn handle_message(&mut self, event: &Message) {
        if let Message::SetInteractionMode(mode) = event {
            match mode {
                InteractionMode::Game => {
                    // Enable
                    if !self.enabled() {
                        self.toggle();
                    }
                }
                InteractionMode::UI | InteractionMode::Block(_) => {
                    // Disable
                    if self.enabled() {
                        self.toggle();
                    }
                }
            }
        }
    }
}
