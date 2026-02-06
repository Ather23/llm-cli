pub struct AgentCore {
    llm: Box<dyn Llm>,
    persistence: LocalPersistence,
}

impl AgentCore {
    pub async fn new(llm: Box<dyn Llm>, persistence: LocalPersistence) -> Self {
        AgentCore { llm, persistence }
    }
}

impl AgentCore {
    pub async fn run(&self, chat_history: Vec<ChatMessage>) -> Result<ChatMessage, Error> {
        let mut chat_history = chat_history;
        let mut chat_message = self.llm.generate_response(&chat_history).await?;
        chat_history.push(chat_message.clone());
        self.persistence.save_chat_history(&chat_history).await?;
        Ok(chat_message)
    }
}
