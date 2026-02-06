use crate::llm_core::ChatMessage;
use async_trait::async_trait;

pub struct Session {
    pub id: String,
}

#[async_trait]
pub trait Persistence {
    fn store_chat_message(&self, message: &ChatMessage) -> Result<(), anyhow::Error>;
    async fn create_session(&mut self) -> Result<Session, anyhow::Error>;
}
