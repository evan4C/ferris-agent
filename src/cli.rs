use clap::{Parser, ValueEnum};

use crate::api::Model;
use crate::conversation::ChatOptions;

/// Ferris Agent — a tool-using LLM chat agent.
#[derive(Debug, Parser)]
#[command(name = "ferris-agent", version, about, long_about = None)]
pub struct Cli {
    /// The user prompt to send. If omitted, starts an interactive REPL session.
    pub prompt: Option<String>,

    /// Model to use for the conversation.
    #[arg(short, long, value_enum, default_value_t = ModelArg::Flash)]
    pub model: ModelArg,

    /// Maximum number of tokens to generate in the response.
    #[arg(long)]
    pub max_tokens: Option<u32>,

    /// Enable streaming responses.
    #[arg(long, default_value_t = false)]
    pub stream: bool,
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
    pub fn chat_options(&self) -> ChatOptions {
        ChatOptions {
            model: self.model.into(),
            max_tokens: self.max_tokens,
            stream: self.stream,
        }
    }
}
