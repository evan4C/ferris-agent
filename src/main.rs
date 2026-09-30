use anyhow::Result;
use clap::Parser;
use ferris_agent::config::AppConfig;
use ferris_agent::config::credential::CredentialStore;
use ferris_agent::{Cli, CliCommand, ConfigCommand, Conversation, DeepSeekClient};
use std::io::Write;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        return run_config_command(command);
    }

    let config = AppConfig::load()?;
    let credentials = CredentialStore::new();
    let api_key = credentials.get_api_key("deepseek")?;
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

fn run_config_command(command: CliCommand) -> Result<()> {
    match command {
        CliCommand::Init => {
            let path = AppConfig::initialize()?;
            println!("Configuration initialized at {}", path.display());
        }
        CliCommand::Config { action } => {
            let credentials = CredentialStore::new();
            match action {
                ConfigCommand::Set { key, value } if key.ends_with(".api_key") => {
                    let provider = key.strip_suffix(".api_key").unwrap_or_default();
                    credentials.set_api_key(provider, &value)?;
                    println!("API key stored in the OS credential manager.");
                }
                ConfigCommand::Set { key, value } => {
                    let path = AppConfig::config_path()?;
                    let mut config = AppConfig::load_from(&path)?;
                    config.set_value(&key, &value)?;
                    config.save_to(&path)?;
                    println!("Configuration updated.");
                }
                ConfigCommand::Get { key } if key.ends_with(".api_key") => {
                    let provider = key.strip_suffix(".api_key").unwrap_or_default();
                    match credentials.get_api_key(provider) {
                        Ok(_) => println!("API key is configured."),
                        Err(ferris_agent::config::credential::CredentialError::NotFound) => {
                            println!("API key is not configured.")
                        }
                        Err(error) => return Err(error.into()),
                    }
                }
                ConfigCommand::Get { key } => {
                    let config = AppConfig::load()?;
                    println!("{}", config.get_value(&key)?);
                }
                ConfigCommand::Delete { key } if key.ends_with(".api_key") => {
                    let provider = key.strip_suffix(".api_key").unwrap_or_default();
                    credentials.delete_api_key(provider)?;
                    println!("API key deleted.");
                }
                ConfigCommand::Delete { .. } => {
                    anyhow::bail!("config delete only supports <provider>.api_key");
                }
            }
        }
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
