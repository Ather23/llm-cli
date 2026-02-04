use async_stream::stream;
use clap::Parser;
use futures::{Stream, StreamExt};
use rig::agent::MultiTurnStreamItem;
use rig::completion::Message;
use rig::prelude::*;
use rig::providers::anthropic;
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::pin::Pin;
use uuid::Uuid;

const LLM_ROOT_DIR: &str = "/home/alif/llm";

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    prompt: String,
}

#[derive(Serialize, Deserialize, Clone)]
enum UserType {
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    user_type: UserType,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let session_path = create_session_dir()?;
    let mut prompt = args.prompt;
    let mut history = Vec::<ChatMessage>::new();

    loop {
        let response_text = {
            let mut stream = generate_response(&history, &prompt);
            let mut response_text = String::new();
            while let Some(response) = stream.next().await {
                match response {
                    LlmResponse::Text(text) => {
                        print!("{}", &text);
                        response_text.push_str(&text);
                    }
                }
            }
            println!(); // Newline after response
            response_text
        };

        // Add to in-memory history
        history.push(ChatMessage {
            user_type: UserType::User,
            message: prompt.clone(),
        });
        history.push(ChatMessage {
            user_type: UserType::Assistant,
            message: response_text.clone(),
        });

        // Save to file
        save_chat_message(&session_path, UserType::User, &prompt)?;
        save_chat_message(&session_path, UserType::Assistant, &response_text)?;

        print!("\n> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;

        prompt = input.trim().to_string();
        if prompt.is_empty() || prompt == "exit" || prompt == "quit" {
            break;
        }
    }

    Ok(())
}

fn create_session_dir() -> Result<String, anyhow::Error> {
    let path = Path::new(LLM_ROOT_DIR);
    if !path.exists() {
        fs::create_dir(LLM_ROOT_DIR).expect("Unable to create session");
    }

    let uuid = Uuid::new_v4();
    let session_path = format!("{}/{}", LLM_ROOT_DIR, uuid);
    fs::create_dir(&session_path)?;

    let file_path = format!("{}/chat.json", &session_path);
    File::create(&file_path)?;
    Ok(file_path)
}

enum LlmResponse {
    Text(String),
}

fn save_chat_message(
    file_path: &str,
    user_type: UserType,
    message: &str,
) -> Result<(), anyhow::Error> {
    // Read existing messages from file
    let mut messages: Vec<ChatMessage> = if Path::new(file_path).exists() {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        // If file is empty or invalid JSON, start with empty vec
        serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };

    // Append new message
    messages.push(ChatMessage {
        user_type,
        message: message.to_string(),
    });

    // Write back to file
    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &messages)?;

    Ok(())
}

fn chat_history_to_messages(chat_history: &[ChatMessage]) -> Vec<Message> {
    chat_history
        .iter()
        .map(|msg| match msg.user_type {
            UserType::User => Message::user(&msg.message),
            UserType::Assistant => Message::assistant(&msg.message),
        })
        .collect()
}

fn generate_response<'a>(
    chat_history: &'a [ChatMessage],
    prompt: &'a str,
) -> Pin<Box<dyn Stream<Item = LlmResponse> + Send + 'a>> {
    let messages = chat_history_to_messages(chat_history);

    Box::pin(stream! {
        let agent = anthropic::Client::from_env()
            .agent(anthropic::completion::CLAUDE_4_OPUS)
            .preamble("Be precise and concise.")
            .temperature(0.5)
            .build();

        let mut stream = agent.stream_chat(prompt, messages).await;
        while let Some(item) = stream.next().await {
            if let Ok(chunk) = item {
                // Only yield text from assistant responses
                if let MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::Text(text)
                ) = chunk {
                    yield LlmResponse::Text(text.text);
                }
            }
        }
    })
}
