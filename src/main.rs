use anyhow::Result;
use clap::Parser;
use ferris_agent::config::AppConfig;
use ferris_agent::config::credential;
use ferris_agent::{Cli, CliCommand, Conversation, DeepSeekClient};
use std::io::Write;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(CliCommand::Init) = cli.command {
        let path = AppConfig::initialize()?;
        println!("Configuration initialized at {}", path.display());
        return Ok(());
    }

    let config = AppConfig::load()?;
    let api_key = credential::get_api_key("deepseek")?;
    let configured_model = config.deepseek.model.clone();
    let max_iterations = config.agent.max_iterations;
    let workspace = config.tools.workspace.clone();
    
    let agent = Arc::new(DeepSeekClient::new(config.deepseek, api_key)?);
    let chat_options = cli.chat_options(&configured_model);
    let mut conversation = Conversation::new(agent).with_max_iterations(max_iterations);
    if let Some(workspace) = workspace {
        conversation = conversation.with_workspace(workspace);
    }
    conversation = conversation.with_options(chat_options);

    match &cli.prompt {
        Some(prompt) => {
            let reply = conversation.send(prompt.clone()).await?;
            println!("{}", reply);
        }
        None => run_repl(&mut conversation).await?,
    }

    Ok(())
}

async fn run_repl(conversation: &mut Conversation) -> Result<()> {
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
            Ok(reply) => println!("{}", reply),
            Err(err) => eprintln!("Error: {}", err),
        }
    }
    Ok(())
}
