use anyhow::Result;
use clap::Parser;
use ferris_agent::{Cli, Conversation, DeepSeekClient};
use std::io::Write;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let agent = Arc::new(DeepSeekClient::new());
    let stream = cli.stream;
    let mut conversation = Conversation::new(agent).with_options(cli.chat_options());

    match &cli.prompt {
        Some(prompt) => {
            let reply = conversation.send(prompt.clone()).await?;
            if !stream {
                println!("{}", reply);
            }
        }
        None => run_repl(&mut conversation, stream).await?,
    }

    Ok(())
}

async fn run_repl(conversation: &mut Conversation, stream: bool) -> Result<()> {
    println!("Ferris Agent interactive session. Type 'exit' or 'quit' to leave.");
    let stdin = std::io::stdin();
    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" || input == "quit" {
            break;
        }

        match conversation.send(input.to_string()).await {
            Ok(reply) => {
                if !stream {
                    println!("{}", reply);
                }
            }
            Err(err) => eprintln!("Error: {}", err),
        }
    }
    Ok(())
}
