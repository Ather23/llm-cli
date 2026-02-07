use clap::Parser;
use futures::StreamExt;
use std::io::{self, BufRead, Write};
mod agent_core;
mod llm_core;
mod persistence;

use agent_core::AgentCore;
use llm_core::{ChatMessage, LlmCore};
use persistence::LocalPersistence;

const LLM_ROOT_DIR: &str = "/home/alif/llm";

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let persistence = LocalPersistence::new(LLM_ROOT_DIR);
    let mut agent = AgentCore::new(LlmCore::new(), persistence).await;

    let mut prompt = args.prompt;

    loop {
        let mut stream = agent.run(&prompt).await;
        while let Some(chat_msg) = stream.next().await {
            match chat_msg {
                ChatMessage::UserMessage(text) | ChatMessage::AssistantMessage(text) => {
                    print!("{}", text);
                }
                ChatMessage::ToolCall(tc) => {
                    print!("[Tool Call: {}]", tc.name);
                }
            }
        }
        println!();

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
