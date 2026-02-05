use crate::llm_core::ChatMessage;

pub trait Persistence {
    fn store_chat_message(&self, message: &ChatMessage) -> Result<(), anyhow::Error>;
}
