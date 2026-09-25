pub mod client;
pub mod models;
pub mod error;

pub use client::DeepSeekClient;
pub use models::{RequestBody, ChatResponse, Model};