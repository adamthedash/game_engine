use crate::{player::Player, world::World};

/// Holds state information about the game independent of the rendering
pub struct GameState {
    player: Player,
    world: World,
}
