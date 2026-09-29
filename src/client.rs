use crate::config::SETTINGS;
use crate::error::DeepSeekError;
use crate::models::*;
use crate::tool::{ToolContext, registry::ToolRegistry};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use std::collections::HashMap;

pub struct DeepSeekClient {
    pub max_steps: u8,
    client: reqwest::Client,
    tool_registry: ToolRegistry,
}

impl DeepSeekClient {
    pub fn new() -> Self {
        Self::with_tool_registry(ToolRegistry::builder().filesystem().build())
    }

    pub fn with_tool_registry(tool_registry: ToolRegistry) -> Self {
        DeepSeekClient {
            max_steps: SETTINGS.deepseek.max_steps,
            client: reqwest::Client::new(),
            tool_registry,
        }
    }

    pub async fn query_llm(
        &self,
        request_body: &ChatCompletionRequest<'_>,
    ) -> Result<String, DeepSeekError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", SETTINGS.deepseek.api_key)).unwrap(),
        );

        let mut payload = serde_json::to_value(request_body)?;
        let mut messages = request_body.messages.clone();
        let mut tool_rounds = 0;

        loop {
            payload["messages"] = serde_json::to_value(&messages)?;

            // Insert the tool call results from the previous round into the messages
            let response = self
                .client
                .post(&SETTINGS.deepseek.base_url)
                .headers(headers.clone())
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(DeepSeekError::Api(format!(
                    "API request failed with status {}: {}",
                    status, text
                )));
            }

            let response = response.json::<ChatResponse>().await?;
            let message = response
                .choices
                .first()
                .map(|choice| &choice.message)
                .ok_or_else(|| DeepSeekError::Api("LLM response contained no choices".into()))?;

            if message.tool_calls.is_empty() {
                return Ok(message.content.clone().unwrap_or_default());
            }
            if tool_rounds >= self.max_steps {
                return Err(DeepSeekError::Api(
                    "Maximum tool-call rounds reached".into(),
                ));
            }
            tool_rounds += 1;

            // Prepare the tool calls for execution
            let calls = message.tool_calls.clone();
            messages.push(Message::assistant_tool_calls(
                message.content.clone(),
                calls.clone(),
            ));
            let working_dir = std::env::current_dir().map_err(|error| {
                DeepSeekError::Api(format!("Cannot determine working directory: {error}"))
            })?;
            let context =
                ToolContext::new(working_dir, std::env::vars().collect::<HashMap<_, _>>());
            let results = self
                .tool_registry
                .dispatch_many(&context, &calls)
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

    pub(crate) fn tool_definitions(&self) -> Vec<crate::tool::registry::ToolDefinition> {
        self.tool_registry.definitions()
    }

    pub fn chat(&self) -> ChatCompletionRequest<'_> {
        ChatCompletionRequest::new(self)
    }
}
