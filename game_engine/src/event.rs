use std::{
    collections::VecDeque,
    sync::{LazyLock, Mutex},
};

use crate::{ui::inventory::ItemFavouritedMessage, world::BlockChangedMessage};

#[derive(Debug)]
#[non_exhaustive]
pub enum Message {
    ItemFavourited(ItemFavouritedMessage),
    BlockChanged(BlockChangedMessage),
}

pub type MessageQueue = VecDeque<Message>;

pub static MESSAGE_QUEUE: LazyLock<Mutex<MessageQueue>> =
    LazyLock::new(|| Mutex::new(MessageQueue::new()));

pub trait Subscriber {
    fn handle_message(&mut self, event: &Message);
}
