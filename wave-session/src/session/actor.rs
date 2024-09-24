#![allow(unused)]

use super::*;
use crate::message::{content::Content, Message};
use async_channel::{Receiver, Sender};
use chrono::Utc;
use wave_core::author::Author;

pub struct Actor {
    author: Author,
    receiver: Receiver<Message>,
    sender: Sender<Message>,
}

impl Actor {
    pub fn new(author: Author, receiver: Receiver<Message>, sender: Sender<Message>) -> Self {
        Self {
            author,
            receiver,
            sender,
        }
    }

    pub async fn send(&self, content: Content) {
        let msg = Message::new(self.author.clone(), content, Utc::now().timestamp() as u64);
        self.sender.send(msg).await.unwrap();
    }

    pub async fn receive(&self) -> Message {
        self.receiver.recv().await.unwrap()
    }
}
