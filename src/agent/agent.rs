use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::api::{ChatUsage, DeepSeekClient, DeepSeekError, Message, Model};
use crate::cli::CliOptions;
use crate::tool::{ToolContext, registry::ToolRegistry};

/// The final assistant reply for a `run`, plus the token usage billed across all rounds.
pub struct AgentReply {
    pub content: String,
    pub usage: Option<ChatUsage>,
}

pub struct Agent {
    client: Arc<DeepSeekClient>,
    messages: Vec<Message>,
    registry: ToolRegistry,
    context: ToolContext,
    options: CliOptions,
    max_iterations: u8,
}

impl Agent {
    pub fn new(client: Arc<DeepSeekClient>) -> Self {
        let working_dir = std::env::current_dir().unwrap_or_else(|_| ".".into());
        Self {
            client,
            messages: vec![Message::system(crate::config::constants::SYSTEM_PROMPT)],
            registry: ToolRegistry::builder().filesystem().build(),
            context: ToolContext::new(working_dir, std::env::vars().collect::<HashMap<_, _>>()),
            options: CliOptions::default(),
            max_iterations: 10,
        }
    }

    pub fn add_user_message(&mut self, content: String) -> &mut Self {
        self.messages.push(Message::user(content));
        self
    }

    pub fn with_max_iterations(&mut self, max_iterations: u8) -> &mut Self {
        self.max_iterations = max_iterations;
        self
    }

    pub fn with_workspace(&mut self, workspace: PathBuf) -> &mut Self {
        self.context.working_dir = workspace;
        self
    }

    pub fn with_tool_registry(&mut self, registry: ToolRegistry) -> &mut Self {
        self.registry = registry;
        self
    }

    pub fn with_options(&mut self, options: CliOptions) -> &mut Self {
        self.options = options;
        self
    }

    /// Read-only view of the accumulated history.
    pub fn history(&self) -> &[Message] {
        &self.messages
    }

    pub fn get_model(&self) -> &Model {
        &self.options.model
    }

    /// Drops all turns, keeping only the initial system prompt.
    pub fn reset_history(&mut self) {
        self.messages
            .truncate(if self.messages.is_empty() { 0 } else { 1 });
    }

    /// main agent loop that handles tool calls and responses.
    pub async fn run(&mut self) -> Result<AgentReply, DeepSeekError> {
        let mut tool_rounds = 0;
        let mut total_usage: Option<ChatUsage> = None;
        let mut request = self
            .client
            .chat()
            .model(self.options.model.clone())
            .stream(self.options.stream)
            .thinking(self.options.thinking)
            .tools(self.registry.definitions());

        if let Some(max_tokens) = self.options.max_tokens {
            request = request.max_tokens(max_tokens);
        }

        loop {
            request = request.messages(self.messages.clone());
            let turn = self.client.http_request(&request).await?;
            if let Some(usage) = turn.usage {
                match &mut total_usage {
                    Some(acc) => *acc += usage,
                    None => total_usage = Some(usage),
                }
            }

            if turn.message.tool_calls.is_empty() {
                let reply = turn.message.content.unwrap_or_default();
                self.messages.push(Message::assistant(reply.clone()));
                return Ok(AgentReply {
                    content: reply,
                    usage: total_usage,
                });
            }

            if tool_rounds >= self.max_iterations {
                return Err(DeepSeekError::Api(
                    "Maximum tool-call rounds reached".into(),
                ));
            }
            tool_rounds += 1;

            self.messages.push(Message::assistant_tool_calls(
                turn.message.content,
                turn.message.tool_calls.clone(),
            ));
            let results = self
                .registry
                .dispatch_many(&self.context, &turn.message.tool_calls)
                .await
                .map_err(|error| DeepSeekError::Api(format!("Tool execution failed: {error}")))?;

            for (tool_call_id, result) in results {
                let content = if result.is_error {
                    format!("Tool failed: {}", result.content)
                } else {
                    result.content
                };
                self.messages
                    .push(Message::tool_result(tool_call_id, content));
            }
        }
    }
}
