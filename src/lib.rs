pub mod client;
pub mod models;
pub mod error;
pub mod config;

pub use client::DeepSeekClient;
pub use models::{RequestBody, ChatResponse, Model};
pub use config::SYSTEM_PROMPT;