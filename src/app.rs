use anyhow::Result;
use std::io::Write;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::config::credential::get_api_key;
use crate::{Cli, CliCommand, Conversation, DeepSeekClient};

pub async fn run(cli: Cli) -> Result<()> {
    if let Some(CliCommand::Init) = cli.command {
        let path = AppConfig::initialize()?;
        println!("Configuration initialized at {}", path.display());
        return Ok(());
    }

    let config = AppConfig::load()?;
    let api_key = get_api_key("deepseek")?;
    let chat_options = cli.chat_options(&config.deepseek.model);
    let mut conversation =
        Conversation::new(Arc::new(DeepSeekClient::new(config.deepseek, api_key)?))
            .with_max_iterations(config.agent.max_iterations);
    if let Some(workspace) = config.tools.workspace {
        conversation = conversation.with_workspace(workspace);
    }
    conversation = conversation.with_options(chat_options);

    match &cli.prompt {
        Some(prompt) => {
            let reply = conversation.send(prompt.clone()).await?;
            println!("{reply}");
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
            Ok(reply) => println!("{reply}"),
            Err(err) => eprintln!("Error: {err}"),
        }
    }
    Ok(())
}
