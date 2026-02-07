use futures::{Stream, StreamExt};
use rig::agent::MultiTurnStreamItem;
use rig::prelude::*;
use rig::providers::anthropic;
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use std::pin::Pin;

use crate::llm_core::core::{ChatMessage, Llm, LlmResponse};

pub struct LlmCore;

impl LlmCore {
    pub fn new() -> Self {
        LlmCore
    }
}

impl Llm for LlmCore {
    fn generate_response(
        &self,
        chat_history: Vec<ChatMessage>,
        prompt: String,
    ) -> Pin<Box<dyn Stream<Item = LlmResponse> + Send>> {
        let messages: Vec<_> = chat_history
            .iter()
            .map(|msg| msg.to_rig_message())
            .collect();

        Box::pin(async_stream::stream! {
            let agent = anthropic::Client::from_env()
                .agent(anthropic::completion::CLAUDE_4_OPUS)
                .preamble("Be precise and concise.")
                .temperature(0.5)
                .build();

            let mut stream = agent.stream_chat(&prompt, messages).await;
            while let Some(item) = stream.next().await {
                if let Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::Text(text)
                )) = item {
                    yield LlmResponse::Text(text.text);
                }
            }
        })
    }
}
