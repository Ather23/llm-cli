use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::llm_core::ChatMessage;
use crate::persistence::core::Persistence;

pub struct LocalPersistence {
    session_path: String,
}

impl LocalPersistence {
    pub fn new(session_path: &str) -> Self {
        LocalPersistence {
            session_path: session_path.to_string(),
        }
    }
}

impl Persistence for LocalPersistence {
    fn store_chat_message(&self, message: &ChatMessage) -> Result<(), anyhow::Error> {
        // Read existing messages from file
        let mut messages: Vec<ChatMessage> = if Path::new(&self.session_path).exists() {
            let file = File::open(&self.session_path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        // Append new message
        messages.push(message.clone());

        // Write back to file
        let file = File::create(&self.session_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &messages)?;

        Ok(())
    }
}
