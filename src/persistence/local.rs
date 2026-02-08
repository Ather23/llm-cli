use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::agent_core::AgentMessage;
use crate::persistence::core::Persistence;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TimestampedMessage {
    timestamp: DateTime<Utc>,
    message: AgentMessage,
}

pub struct LocalPersistence {
    root_dir: String,
}

impl LocalPersistence {
    pub fn new(root_dir: &str) -> Self {
        LocalPersistence {
            root_dir: root_dir.to_string(),
        }
    }
}

#[async_trait]
impl Persistence for LocalPersistence {
    async fn store_chat_message(
        &self,
        message: &AgentMessage,
        session_id: &str,
    ) -> Result<(), anyhow::Error> {
        let session_path = format!("{}/{}/chat.json", self.root_dir, session_id);

        // Read existing messages from file
        let mut messages: Vec<TimestampedMessage> = if Path::new(&session_path).exists() {
            let file = File::open(&session_path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        // Append new message with timestamp
        let timestamped = TimestampedMessage {
            timestamp: Utc::now(),
            message: message.clone(),
        };
        messages.push(timestamped);

        // Create parent directories if they don't exist
        let parent_dir = Path::new(&session_path).parent().unwrap();
        fs::create_dir_all(parent_dir)?;

        // Write back to file
        let file = File::create(session_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &messages)?;

        Ok(())
    }
}
