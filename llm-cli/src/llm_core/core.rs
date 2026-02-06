use futures::Stream;
use rig::completion::Message;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Serialize, Deserialize, Clone)]
pub enum UserType {
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub user_type: UserType,
    pub message: String,
}

impl ChatMessage {
    pub fn to_rig_message(&self) -> Message {
        match self.user_type {
            UserType::User => Message::user(&self.message),
            UserType::Assistant => Message::assistant(&self.message),
        }
    }
}

pub enum LlmResponse {
    Text(String),
}

pub trait Llm {
    fn generate_response(
        &self,
        chat_history: Vec<ChatMessage>,
        prompt: String,
    ) -> Pin<Box<dyn Stream<Item = LlmResponse> + Send>>;
}
