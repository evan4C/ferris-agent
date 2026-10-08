use clap::{Parser, Subcommand, ValueEnum};

use crate::api::Model;

/// Ferris Agent — a tool-using LLM chat agent.
#[derive(Debug, Parser)]
#[command(name = "ferris-agent", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// The user prompt to send. If omitted, starts an interactive REPL session.
    pub prompt: Option<String>,

    /// Model to use for the conversation.
    #[arg(short, long, value_enum)]
    pub model: Option<ModelArg>,

    /// Maximum number of tokens to generate in the response.
    #[arg(long)]
    pub max_tokens: Option<u32>,

    /// Enable streaming responses.
    #[arg(long, default_value_t = false)]
    pub stream: bool,

    /// Enable thinking mode and display the model's reasoning output on stderr.
    #[arg(long, default_value_t = false)]
    pub thinking: bool,
}

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    /// Create the user configuration file.
    Init,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ModelArg {
    Flash,
    Pro,
}

impl From<ModelArg> for Model {
    fn from(value: ModelArg) -> Self {
        match value {
            ModelArg::Flash => Model::Flash,
            ModelArg::Pro => Model::Pro,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CliOptions {
    pub model: Model,
    pub max_tokens: Option<u32>,
    pub stream: bool,
    pub thinking: bool,
}

impl CliOptions {
    pub fn default() -> Self {
        Self {
            model: Model::Flash,
            max_tokens: None,
            stream: false,
            thinking: false,
        }
    }
}

impl Cli {
    pub fn cli_options(&self, config_toml_model: &str) -> CliOptions {
        let configured_model = if config_toml_model.contains("pro") {
            Model::Pro
        } else {
            Model::Flash
        };
        CliOptions {
            model: self
                .model
                .map(Into::into)
                .unwrap_or(configured_model),
            max_tokens: self.max_tokens,
            stream: self.stream,
            thinking: self.thinking,
        }
    }
}
