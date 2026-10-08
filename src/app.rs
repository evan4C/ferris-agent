use anyhow::Result;
use std::io::Write;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::config::credential::get_api_key;
use crate::{Cli, CliCommand, Agent, DeepSeekClient};

pub async fn exec(cli: Cli) -> Result<()> {
    if let Some(CliCommand::Init) = cli.command {
        let path = AppConfig::initialize()?;
        println!("Configuration initialized at {}", path.display());
        return Ok(());
    }

    // load app configuration from config.toml
    let config = AppConfig::load()?;
    // load API key from environment variable
    let api_key = get_api_key("deepseek")?;
    // load chat options from cli parameters
    let cli_options = cli.cli_options(&config.deepseek.model);

    let mut agent = Agent::new(Arc::new(DeepSeekClient::new(config.deepseek, api_key)?));

    agent.with_max_iterations(config.agent.max_iterations);
    if let Some(workspace) = config.tools.workspace {
        agent.with_workspace(workspace);
    }
    agent.with_options(cli_options);

    match &cli.prompt {
        Some(prompt) => {
            agent.add_user_message(prompt.clone());
            let reply = agent.run().await?;
            println!("{reply}");
        }
        None => run_repl(&mut agent).await?,
    }

    Ok(())
}

async fn run_repl(agent: &mut Agent) -> Result<()> {
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

        match agent.add_user_message(input.to_string()).run().await {
            Ok(reply) => println!("{reply}"),
            Err(err) => eprintln!("Error: {err}"),
        }
    }
    Ok(())
}
