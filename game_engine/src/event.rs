use std::{
    collections::VecDeque,
    sync::{LazyLock, Mutex},
};

use crate::{state::world::BlockChangedMessage, ui::inventory::ItemFavouritedMessage, InteractionMode};

#[derive(Debug)]
pub enum Message {
    // Reactive messages - This thing has happened
    ItemFavourited(ItemFavouritedMessage),
    BlockChanged(BlockChangedMessage),

    // Action messages - Do this thing
    SetInteractionMode(InteractionMode),
}

pub type MessageQueue = VecDeque<Message>;

pub static MESSAGE_QUEUE: LazyLock<Mutex<MessageQueue>> =
    LazyLock::new(|| Mutex::new(MessageQueue::new()));

pub trait Subscriber {
    fn handle_message(&mut self, event: &Message);
}
