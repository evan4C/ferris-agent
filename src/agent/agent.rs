use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::api::{DeepSeekClient, DeepSeekError, Message};
use crate::conversation::ChatOptions;
use crate::tool::{ToolContext, registry::ToolRegistry};

pub struct Agent {
    client: Arc<DeepSeekClient>,
    registry: ToolRegistry,
    context: ToolContext,
    options: ChatOptions,
    max_iterations: u8,
}

impl Agent {
    pub fn new(client: Arc<DeepSeekClient>) -> Self {
        let working_dir = std::env::current_dir().unwrap_or_else(|_| ".".into());
        Self {
            client,
            registry: ToolRegistry::builder().filesystem().build(),
            context: ToolContext::new(working_dir, std::env::vars().collect::<HashMap<_, _>>()),
            options: ChatOptions::default(),
            max_iterations: 10,
        }
    }

    pub fn with_max_iterations(mut self, max_iterations: u8) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    pub fn with_workspace(mut self, workspace: PathBuf) -> Self {
        self.context.working_dir = workspace;
        self
    }

    pub fn with_tool_registry(mut self, registry: ToolRegistry) -> Self {
        self.registry = registry;
        self
    }

    pub fn with_options(mut self, options: ChatOptions) -> Self {
        self.options = options;
        self
    }

    pub async fn run(&self, messages: &mut Vec<Message>) -> Result<String, DeepSeekError> {
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
            request = request.messages(messages.clone());
            let response = self
                .client
                .http_request(&request)
                .await?;
            if response.tool_calls.is_empty() {
                let reply = response.content.unwrap_or_default();
                messages.push(Message::assistant(reply.clone()));
                return Ok(reply);
            }

            if tool_rounds >= self.max_iterations {
                return Err(DeepSeekError::Api(
                    "Maximum tool-call rounds reached".into(),
                ));
            }
            tool_rounds += 1;

            messages.push(Message::assistant_tool_calls(
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
                messages.push(Message::tool_result(tool_call_id, content));
            }
        }
    }
}
