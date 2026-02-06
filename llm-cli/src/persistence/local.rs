use async_trait::async_trait;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use uuid::Uuid;

use crate::llm_core::ChatMessage;
use crate::persistence::core::{Persistence, Session};

pub struct LocalPersistence {
    root_dir: String,
    session_path: Option<String>,
}

impl LocalPersistence {
    pub fn new(root_dir: &str) -> Self {
        LocalPersistence {
            root_dir: root_dir.to_string(),
            session_path: None,
        }
    }
}

#[async_trait]
impl Persistence for LocalPersistence {
    fn store_chat_message(&self, message: &ChatMessage) -> Result<(), anyhow::Error> {
        let session_path = self
            .session_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No session created. Call create_session() first."))?;

        // Read existing messages from file
        let mut messages: Vec<ChatMessage> = if Path::new(session_path).exists() {
            let file = File::open(session_path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        // Append new message
        messages.push(message.clone());

        // Write back to file
        let file = File::create(session_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &messages)?;

        Ok(())
    }

    async fn create_session(&mut self) -> Result<Session, anyhow::Error> {
        // Create root dir if needed
        let path = Path::new(&self.root_dir);
        if !path.exists() {
            fs::create_dir(&self.root_dir)?;
        }

        // Create session directory
        let id = Uuid::new_v4().to_string();
        let session_dir = format!("{}/{}", self.root_dir, id);
        fs::create_dir(&session_dir)?;

        // Create chat.json file
        let file_path = format!("{}/chat.json", session_dir);
        File::create(&file_path)?;

        // Store for later use
        self.session_path = Some(file_path);

        Ok(Session { id })
    }
}
