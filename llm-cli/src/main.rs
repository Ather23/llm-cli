use clap::Parser;
use futures::StreamExt;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;
use uuid::Uuid;

mod llm_core;
mod persistence;

use llm_core::{ChatMessage, Llm, LlmCore, LlmResponse, UserType};
use persistence::{LocalPersistence, Persistence};

const LLM_ROOT_DIR: &str = "/home/alif/llm";

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let session_path = create_session_dir()?;
    let persistence = LocalPersistence::new(&session_path);
    let mut prompt = args.prompt;
    let mut history = Vec::<ChatMessage>::new();
    let llm = LlmCore::new();

    loop {
        let response_text = {
            let mut stream = llm.generate_response(&history, &prompt);
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
        persistence.store_chat_message(&ChatMessage {
            user_type: UserType::User,
            message: prompt.clone(),
        })?;
        persistence.store_chat_message(&ChatMessage {
            user_type: UserType::Assistant,
            message: response_text.clone(),
        })?;

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
