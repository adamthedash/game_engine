use std::{ collections::VecDeque};

use enum_array::{
    EnumArray, enum_array_derive::EnumDiscriminant, enum_array_trait,
};

#[derive(Debug)]
struct BlockBroken {}

#[derive(Debug)]
struct BlockPlaced {}

#[derive(Debug, EnumDiscriminant)]
enum Message {
    BlockBroken(BlockBroken),
    BlockPlaced(BlockPlaced),
}

struct MessageQueue {
    queue: VecDeque<Message>,
    subscribers: EnumArray<Vec<Box<dyn Subscriber>>, Message>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            subscribers: EnumArray(std::array::from_fn(|_| vec![])),
        }
    }

    pub fn process_messages(&mut self) {
        while let Some(message) = self.queue.pop_front() {
            self.subscribers[&message]
                .iter_mut()
                .for_each(|subscriber| {
                    subscriber.handle(&message);
                });
        }
    }
}

pub trait Subscriber {
    fn handle(&mut self, event: &Message);
}

#[cfg(test)]
mod tests {
    use crate::event::{BlockBroken, BlockPlaced, Message, MessageQueue, Subscriber};


    #[test]
    fn test_queue(){
        struct Handler1;
        impl Subscriber for Handler1 {
            fn handle(&mut self, event: &super::Message) {
                println!("{:?}", event);
            }
        }

        let mut queue = MessageQueue::new();
        queue.subscribers[&Message::BlockBroken(BlockBroken {  })].push(Box::new(Handler1));

        queue.queue.push_back(Message::BlockBroken(BlockBroken{}));
        queue.queue.push_back(Message::BlockPlaced(BlockPlaced{}));

        queue.process_messages();

    }
}
