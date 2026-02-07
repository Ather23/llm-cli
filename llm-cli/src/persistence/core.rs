use crate::llm_core::ChatMessage;
use async_trait::async_trait;

#[async_trait]
pub trait Persistence {
    async fn store_chat_message(
        &self,
        message: &ChatMessage,
        session_id: &str,
    ) -> Result<(), anyhow::Error>;
}
