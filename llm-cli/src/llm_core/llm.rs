use async_stream::stream;
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
    fn generate_response<'a>(
        &'a self,
        chat_history: &'a [ChatMessage],
        prompt: &'a str,
    ) -> Pin<Box<dyn Stream<Item = LlmResponse> + Send + 'a>> {
        let messages: Vec<_> = chat_history
            .iter()
            .map(|msg| msg.to_rig_message())
            .collect();

        Box::pin(stream! {
            let agent = anthropic::Client::from_env()
                .agent(anthropic::completion::CLAUDE_4_OPUS)
                .preamble("Be precise and concise.")
                .temperature(0.5)
                .build();

            let mut stream = agent.stream_chat(prompt, messages).await;
            while let Some(item) = stream.next().await {
                if let Ok(chunk) = item {
                    if let MultiTurnStreamItem::StreamAssistantItem(
                        StreamedAssistantContent::Text(text)
                    ) = chunk {
                        yield LlmResponse::Text(text.text);
                    }
                }
            }
        })
    }
}
