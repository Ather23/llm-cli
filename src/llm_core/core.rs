use futures::Stream;
use rig::completion::Message;
use std::pin::Pin;

#[derive(Clone)]
pub enum LlmMessage {
    UserMessage(String),
    AssistantMessage(String),
    ToolCall(ToolCall),
}

#[derive(Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

impl LlmMessage {
    pub fn to_rig_message(&self) -> Message {
        match self {
            LlmMessage::UserMessage(text) => Message::user(text),
            LlmMessage::AssistantMessage(text) => Message::assistant(text),
            LlmMessage::ToolCall(tc) => Message::assistant(format!("[Tool Call: {}]", tc.name)),
        }
    }
}

pub trait Llm {
    fn generate_response(
        &self,
        chat_history: Vec<LlmMessage>,
        prompt: String,
    ) -> Pin<Box<dyn Stream<Item = LlmMessage> + Send>>;
}
