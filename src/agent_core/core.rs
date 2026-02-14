use crate::agent_core::session::Session;
use crate::llm_core::{Llm, LlmMessage};
use crate::persistence::Persistence;

use async_stream::stream;
use async_trait::async_trait;
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[async_trait]
pub trait EventListener: Send + Sync {
    async fn on_stream_start(&self);
    async fn on_user_message(&self, text: &str);
    async fn on_assistant_message(&self, text: &str);
    async fn on_tool_call(&self, tool_call: &ToolCall);
    async fn on_stream_end(&self);
}

pub struct AgentEvents;

#[async_trait]
impl EventListener for AgentEvents {
    async fn on_stream_start(&self) {
        println!("stream started");
    }

    async fn on_user_message(&self, _text: &str) {
        // Stub: Handle user message
        println!(">> user message received: {}", _text);
    }

    async fn on_assistant_message(&self, _text: &str) {
        // Stub: Handle assistant message
        println!("### assistant message received: {}", _text);
    }

    async fn on_tool_call(&self, _tool_call: &ToolCall) {
        // Stub: Handle tool call
        println!("xxx tool call received: {:?}", &_tool_call);
    }

    async fn on_stream_end(&self) {
        // Stub: Handle stream end
        println!("stream ended");
    }
}

pub struct AgentCore<L: Llm + 'static, P: Persistence + 'static> {
    pub llm: L,
    pub persistence: P,
    pub session: Session,
    pub event_listeners: Vec<Box<dyn EventListener>>,
    pub chat_history: Vec<AgentMessage>,
}

impl<L, P> AgentCore<L, P>
where
    L: Llm + Send,
    P: Persistence + Send,
{
    pub async fn new(llm: L, persistence: P, event_listeners: Vec<Box<dyn EventListener>>) -> Self {
        let session = Session::new();
        let chat_history = persistence.load_context().await.unwrap();

        AgentCore {
            llm,
            persistence,
            session,
            chat_history: chat_history,
            event_listeners,
        }
    }

    pub async fn run(
        &mut self,
        user_message: &str,
    ) -> Pin<Box<dyn Stream<Item = AgentMessage> + Send + '_>> {
        for listener in &self.event_listeners {
            listener.on_stream_start().await;
        }

        for listener in &self.event_listeners {
            listener.on_user_message(user_message).await;
        }

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

                        for listener in &self.event_listeners {
                            listener.on_assistant_message(&text).await;
                        }

                        yield AgentMessage::AssistantMessage(text);
                    }
                    LlmMessage::ToolCall(tc) => {
                        let tool_call: ToolCall = tc.into();

                        for listener in &self.event_listeners {
                            listener.on_tool_call(&tool_call).await;
                        }

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

            for listener in &self.event_listeners {
                listener.on_stream_end().await;
            }
        })
    }
}
