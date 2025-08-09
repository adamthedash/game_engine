pub mod messages;

use std::{
    collections::VecDeque,
    sync::{LazyLock, Mutex},
};

use crate::{
    InteractionMode,
    entity::components::UprightOrientation,
    event::messages::{
        BlockChangedMessage, ItemFavouritedMessage, PlaceBlockMessage, SetCraftingRecipeMessage,
        SpawnEntityMessage, TransferItemMessage, TransferItemRequestMessage,
    },
    state::world::{BlockPos, WorldPos},
};

#[derive(Debug)]
pub enum Message {
    // Reactive messages - This thing has happened
    ItemFavourited(ItemFavouritedMessage),
    BlockChanged(BlockChangedMessage),
    PlayerMoved((WorldPos, UprightOrientation)),

    // Action messages - Do this thing
    // It's assumed that at the action has been validated at this point
    ToggleInteractionMode,
    SetInteractionMode(InteractionMode),
    BreakBlock(BlockPos),
    PlaceBlock(PlaceBlockMessage),
    SetCraftingRecipe(SetCraftingRecipeMessage),

    // Transfer an item from the player's inventory to whatever interface is open
    TransferItemRequest(TransferItemRequestMessage),
    TransferItem(TransferItemMessage),

    SpawnEntity(SpawnEntityMessage),
}

pub struct MessageQueue {
    queue: Mutex<VecDeque<Message>>,
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub fn send(&self, message: Message) {
        self.queue
            .lock()
            .expect("Failed to lock message queue")
            .push_back(message);
    }

    pub fn take(&self) -> Option<Message> {
        self.queue
            .lock()
            .expect("Failed to lock message queue")
            .pop_front()
    }
}

pub static MESSAGE_QUEUE: LazyLock<MessageQueue> = LazyLock::new(MessageQueue::new);

pub trait Subscriber {
    fn handle_message(&mut self, event: &Message);
}
