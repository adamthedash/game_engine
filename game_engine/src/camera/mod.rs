use std::{f32::consts::PI, time::Duration};

use hecs::Entity;
use winit::event::KeyEvent;

use crate::{
    InteractionMode,
    camera::{
        basic_flight::BasicFlightController, space_flight::SpaceFlightController,
        traits::PlayerController, walking::WalkingController,
    },
    entity::components::UprightOrientation,
    event::{MESSAGE_QUEUE, Message, Subscriber},
    state::world::{World, WorldPos},
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

    fn handle_mouse_move(
        &mut self,
        ecs: &mut hecs::World,
        entity: Entity,
        delta: (f32, f32),
    ) -> bool {
        let moved = self
            .as_controller_mut()
            .handle_mouse_move(ecs, entity, delta);

        let mut query = ecs
            .query_one::<(&mut WorldPos, &mut UprightOrientation)>(entity)
            .unwrap();
        let (position, orientation) = query.get().unwrap();
        if moved {
            MESSAGE_QUEUE.send(Message::PlayerMoved((
                *position,
                orientation.clone(),
            )));
        }

        moved
    }

    fn move_entity(
        &mut self,
        ecs: &mut hecs::World,
        entity: Entity,
        world: &World,
        duration: &Duration,
    ) -> bool {
        let moved = self
            .as_controller_mut()
            .move_entity(ecs, entity, world, duration);

        let mut query = ecs
            .query_one::<(&mut WorldPos, &mut UprightOrientation)>(entity)
            .unwrap();
        let (position, orientation) = query.get().unwrap();
        if moved {
            MESSAGE_QUEUE.send(Message::PlayerMoved((
                *position,
                orientation.clone(),
            )));
        }

        moved
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
