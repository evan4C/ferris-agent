use clap::{Parser, Subcommand, ValueEnum};

use crate::api::Model;
use crate::conversation::ChatOptions;

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

    /// Show the model's reasoning (thinking) output on stderr.
    #[arg(long, default_value_t = false)]
    pub show_reasoning: bool,
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

impl Cli {
    pub fn chat_options(&self, configured_model: &str) -> ChatOptions {
        ChatOptions {
            model: self
                .model
                .map(Into::into)
                .unwrap_or_else(|| Model::Custom(configured_model.into())),
            max_tokens: self.max_tokens,
            stream: self.stream,
            show_reasoning: self.show_reasoning,
        }
    }
}
