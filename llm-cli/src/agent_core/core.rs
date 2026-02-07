use crate::agent_core::session::Session;
use crate::llm_core::{Llm, LlmMessage};
use crate::persistence::Persistence;

use async_stream::stream;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum AgentMessage {
    UserMessage(String),
    AssistantMessage(String),
    ToolCall(ToolCall),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

impl From<&AgentMessage> for LlmMessage {
    fn from(msg: &AgentMessage) -> Self {
        match msg {
            AgentMessage::UserMessage(text) => LlmMessage::UserMessage(text.clone()),
            AgentMessage::AssistantMessage(text) => LlmMessage::AssistantMessage(text.clone()),
            AgentMessage::ToolCall(tc) => LlmMessage::ToolCall(tc.into()),
        }
    }
}

impl From<&ToolCall> for crate::llm_core::ToolCall {
    fn from(tc: &ToolCall) -> Self {
        crate::llm_core::ToolCall {
            id: tc.id.clone(),
            name: tc.name.clone(),
            arguments: tc.arguments.clone(),
        }
    }
}

impl From<crate::llm_core::ToolCall> for ToolCall {
    fn from(tc: crate::llm_core::ToolCall) -> Self {
        ToolCall {
            id: tc.id,
            name: tc.name,
            arguments: tc.arguments,
        }
    }
}

pub struct AgentCore<L: Llm + 'static, P: Persistence + 'static> {
    pub llm: L,
    pub persistence: P,
    pub session: Session,
    pub chat_history: Vec<AgentMessage>,
}

impl<L, P> AgentCore<L, P>
where
    L: Llm,
    P: Persistence,
{
    pub async fn new(llm: L, persistence: P) -> Self {
        AgentCore {
            llm,
            persistence,
            session: Session::new(),
            chat_history: Vec::new(),
        }
    }
}

impl<L, P> AgentCore<L, P>
where
    L: Llm + Send,
    P: Persistence + Send,
{
    pub async fn run(
        &mut self,
        user_message: &str,
    ) -> Pin<Box<dyn Stream<Item = AgentMessage> + Send + '_>> {
        let user_msg = AgentMessage::UserMessage(user_message.to_string());
        self.chat_history.push(user_msg.clone());
        let _ = self
            .persistence
            .store_chat_message(&user_msg, &self.session.id)
            .await;

        let chat_history: Vec<LlmMessage> = self.chat_history.iter().map(|m| m.into()).collect();
        let mut stream = self
            .llm
            .generate_response(chat_history, user_message.to_string());

        Box::pin(stream! {
            let mut full_response = String::new();

            while let Some(response) = stream.next().await {
                match response {
                    LlmMessage::AssistantMessage(text) => {
                        full_response.push_str(&text);
                        yield AgentMessage::AssistantMessage(text);
                    }
                    LlmMessage::ToolCall(tc) => {
                        let tool_call: ToolCall = tc.into();
                        self.chat_history.push(AgentMessage::ToolCall(tool_call.clone()));
                        let _ = self.persistence.store_chat_message(
                            &AgentMessage::ToolCall(tool_call.clone()),
                            &self.session.id
                        ).await;
                        yield AgentMessage::ToolCall(tool_call);
                    }
                    LlmMessage::UserMessage(_) => {
                        // Unexpected from LLM, ignore
                    }
                }
            }

            if !full_response.is_empty() {
                let assistant_msg = AgentMessage::AssistantMessage(full_response);
                self.chat_history.push(assistant_msg.clone());
                let _ = self.persistence.store_chat_message(&assistant_msg, &self.session.id).await;
            }
        })
    }
}
