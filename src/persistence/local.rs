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
pub struct TimestampedMessage {
    timestamp: DateTime<Utc>,
    message: AgentMessage,
}

pub struct LocalPersistence {
    root_dir: String,
    global_persistence: bool,
    session_id: String,
}

impl LocalPersistence {
    pub fn new(root_dir: &str, session_id: &str, use_global_persistence: bool) -> Self {
        LocalPersistence {
            root_dir: root_dir.to_string(),
            session_id: session_id.to_string(),
            global_persistence: use_global_persistence,
        }
    }

    pub fn load_context(&self) -> Result<Vec<TimestampedMessage>, anyhow::Error> {
        let session_path = if self.global_persistence {
            format!("{}/{}/chat.json", self.root_dir, "global")
        } else {
            format!("{}/{}/chat.json", self.root_dir, &self.session_id)
        };

        // Read existing messages from file
        let mut messages: Vec<TimestampedMessage> = if Path::new(&session_path).exists() {
            let file = File::open(session_path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        Ok(messages)
    }
}

#[async_trait]
impl Persistence for LocalPersistence {
    async fn store_chat_message(
        &self,
        message: &AgentMessage,
        session_id: &str,
    ) -> Result<(), anyhow::Error> {
        let session_path = if self.global_persistence {
            format!("{}/{}/chat.json", self.root_dir, "global")
        } else {
            format!("{}/{}/chat.json", self.root_dir, &session_id)
        };

        let mut messages: Vec<TimestampedMessage> = self.load_context().unwrap().clone();

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

    async fn load_context(&self) -> Result<Vec<AgentMessage>, anyhow::Error> {
        let messages = self.load_context().unwrap();
        let agent_messages: Vec<AgentMessage> =
            messages.into_iter().map(|msg| msg.message).collect();
        Ok(agent_messages)
    }
}
