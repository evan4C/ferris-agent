use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::api::{DeepSeekClient, DeepSeekError, Message};
use crate::cli::CliOptions;
use crate::tool::{ToolContext, registry::ToolRegistry};

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

    /// Drops all turns, keeping only the initial system prompt.
    pub fn reset_history(&mut self) {
        self.messages
            .truncate(if self.messages.is_empty() { 0 } else { 1 });
    }

    /// main agent loop that handles tool calls and responses.
    pub async fn run(&mut self) -> Result<String, DeepSeekError> {
        let mut tool_rounds = 0;
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
            let response = self
                .client
                .http_request(&request)
                .await?;
            if response.tool_calls.is_empty() {
                let reply = response.content.unwrap_or_default();
                self.messages.push(Message::assistant(reply.clone()));
                return Ok(reply);
            }

            if tool_rounds >= self.max_iterations {
                return Err(DeepSeekError::Api(
                    "Maximum tool-call rounds reached".into(),
                ));
            }
            tool_rounds += 1;

            self.messages.push(Message::assistant_tool_calls(
                response.content,
                response.tool_calls.clone(),
            ));
            let results = self
                .registry
                .dispatch_many(&self.context, &response.tool_calls)
                .await
                .map_err(|error| DeepSeekError::Api(format!("Tool execution failed: {error}")))?;

            for (tool_call_id, result) in results {
                let content = if result.is_error {
                    format!("Tool failed: {}", result.content)
                } else {
                    result.content
                };
                self.messages.push(Message::tool_result(tool_call_id, content));
            }
        }
    }
}
