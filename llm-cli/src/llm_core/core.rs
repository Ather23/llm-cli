use futures::Stream;
use rig::completion::Message;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Serialize, Deserialize, Clone)]
pub enum ChatMessage {
    UserMessage(String),
    AssistantMessage(String),
    ToolCall(ToolCall),
}

impl ChatMessage {
    pub fn to_rig_message(&self) -> Message {
        match self {
            ChatMessage::UserMessage(text) => Message::user(text),
            ChatMessage::AssistantMessage(text) => Message::assistant(text),
            ChatMessage::ToolCall(tc) => Message::assistant(format!("[Tool Call: {}]", tc.name)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
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
