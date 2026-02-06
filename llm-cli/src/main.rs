use clap::Parser;
use futures::StreamExt;
use std::io::{self, BufRead, Write};

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

    let mut persistence = LocalPersistence::new(LLM_ROOT_DIR);
    persistence.create_session().await?;

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
