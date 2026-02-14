use clap::Parser;
use futures::StreamExt;
use std::io::{self, BufRead, Write};
mod agent_core;
mod llm_core;
mod persistence;

use agent_core::{AgentCore, AgentEvents, AgentMessage, EventListener};
use llm_core::LlmCore;
use persistence::LocalPersistence;

use crate::agent_core::Session;

const LLM_ROOT_DIR: &str = "/home/alif/llm";

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,

    #[arg(short = 'g', long, default_value_t = true)]
    use_global_context: bool,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let session = Session::new();
    let persistence = LocalPersistence::new(LLM_ROOT_DIR, &session.id, args.use_global_context);
    let event_listeners: Vec<Box<dyn EventListener>> = vec![Box::new(AgentEvents)];
    let mut agent = AgentCore::new(LlmCore::new(), persistence, Vec::new()).await;

    let mut prompt = args.prompt;

    loop {
        let mut stream = agent.run(&prompt).await;
        while let Some(chat_msg) = stream.next().await {
            match chat_msg {
                AgentMessage::UserMessage(text) | AgentMessage::AssistantMessage(text) => {
                    print!("{}", text);
                }
                AgentMessage::ToolCall(tc) => {
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
