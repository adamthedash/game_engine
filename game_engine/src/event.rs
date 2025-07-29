use std::{
    collections::VecDeque,
    sync::{LazyLock, Mutex},
};

use crate::{
    InteractionMode,
    state::world::{BlockChangedMessage, BlockPos, PlaceBlockMessage},
    ui::inventory::ItemFavouritedMessage,
};

#[derive(Debug)]
pub enum Message {
    // Reactive messages - This thing has happened
    ItemFavourited(ItemFavouritedMessage),
    BlockChanged(BlockChangedMessage),

    // Action messages - Do this thing
    // It's assumed that at the action has been validated at this point
    SetInteractionMode(InteractionMode),
    BreakBlock(BlockPos),
    PlaceBlock(PlaceBlockMessage),
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
