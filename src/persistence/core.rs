use crate::agent_core::AgentMessage;
use async_trait::async_trait;

#[async_trait]
pub trait Persistence {
    async fn store_chat_message(
        &self,
        message: &AgentMessage,
        session_id: &str,
    ) -> Result<(), anyhow::Error>;
}
