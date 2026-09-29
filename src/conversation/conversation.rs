use std::sync::Arc;

use crate::agent::Agent;
use crate::api::{DeepSeekClient, DeepSeekError, Message};
use crate::config;
use crate::conversation::ChatOptions;
use crate::tool::registry::ToolRegistry;

/// Drives a multi-turn chat session, keeping the full message history
/// (including tool calls and their results) across calls to `send`.
pub struct Conversation {
    agent: Agent,
    messages: Vec<Message>,
}

impl Conversation {
    pub fn new(client: Arc<DeepSeekClient>) -> Self {
        Self {
            agent: Agent::new(client),
            messages: vec![Message::system(config::constants::SYSTEM_PROMPT)],
        }
    }

    pub fn with_tool_registry(mut self, registry: ToolRegistry) -> Self {
        self.agent = self.agent.with_tool_registry(registry);
        self
    }

    pub fn with_options(mut self, options: ChatOptions) -> Self {
        self.agent = self.agent.with_options(options);
        self
    }

    /// Read-only view of the accumulated history.
    pub fn history(&self) -> &[Message] {
        &self.messages
    }

    /// Drops all turns, keeping only the initial system prompt.
    pub fn reset(&mut self) {
        self.messages
            .truncate(if self.messages.is_empty() { 0 } else { 1 });
    }

    /// Sends a user message, resolving any tool calls and returns the final assistant reply
    pub async fn send(&mut self, user_prompt: impl Into<String>) -> Result<String, DeepSeekError> {
        self.messages.push(Message::user(user_prompt));
        self.agent.run(&mut self.messages).await
    }
}
