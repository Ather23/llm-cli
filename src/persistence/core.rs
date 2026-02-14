use crate::agent_core::AgentMessage;
use async_trait::async_trait;
use uuid::Timestamp;

#[async_trait]
pub trait Persistence {
    async fn store_chat_message(
        &self,
        message: &AgentMessage,
        session_id: &str,
    ) -> Result<(), anyhow::Error>;

    async fn load_context(&self) -> Result<Vec<AgentMessage>, anyhow::Error>;
}

// impl From<&TimestampedMessage> for AgentMessage {
//     fn from(msg: &TimestampedMessage) -> Self {
//         match msg {
//             TimestampedMessage::(text) => AgentMessage::UserMessage(text.clone()),
//             TimestampedMessage::AssistantMessage(text) => {
//                 AgentMessage::AssistantMessage(text.clone())
//             }
//             TimestampedMessage::ToolCall(tc) => AgentMessage::ToolCall(tc.into()),
//         }
//     }
// }
