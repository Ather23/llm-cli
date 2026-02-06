use crate::agent_core::session::Session;
use crate::llm_core::{ChatMessage, Llm, LlmResponse, UserType};
use crate::persistence::Persistence;

use async_stream::stream;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use uuid::Uuid;

pub struct AgentCore<L: Llm + 'static, P: Persistence + 'static> {
    pub llm: L,
    pub persistence: P,
    pub session: Session,
    pub chat_history: Vec<ChatMessage>,
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
    ) -> Pin<Box<dyn Stream<Item = ChatMessage> + Send + '_>> {
        let user_msg = ChatMessage {
            user_type: UserType::User,
            message: user_message.to_string(),
        };
        self.chat_history.push(user_msg.clone());
        let _ = self
            .persistence
            .store_chat_message(&user_msg, &self.session.id)
            .await;

        let history = self.chat_history.clone();
        let mut stream = self
            .llm
            .generate_response(history, user_message.to_string());

        Box::pin(stream! {
            while let Some(response) = stream.next().await {
                let LlmResponse::Text(text) = response;
                let assistant_msg = ChatMessage {
                    user_type: UserType::Assistant,
                    message: text.clone(),
                };
                self.chat_history.push(assistant_msg.clone());
                let _ = self.persistence.store_chat_message(&assistant_msg, &self.session.id).await;
                yield assistant_msg;
            }
        })
    }
}
