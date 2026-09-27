use crate::config::SETTINGS;
use crate::error::DeepSeekError;
use crate::models::*;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

#[derive(Debug)]
pub struct DeepSeekClient {
    pub max_retries: u8,
    client: reqwest::Client,
}

impl DeepSeekClient {
    pub fn new() -> Self {
        DeepSeekClient {
            max_retries: SETTINGS.deepseek.max_retries,
            client: reqwest::Client::new(),
        }
    }

    pub async fn query_llm(&self, request_body: &ChatBuilder<'_>) -> Result<String, DeepSeekError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", SETTINGS.deepseek.api_key)).unwrap(),
        );

        let response = self
            .client
            .post(&SETTINGS.deepseek.base_url)
            .headers(headers)
            .json(request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(DeepSeekError::Api(format!(
                "API request failed with status {}: {}",
                status, text
            ))
            .into());
        }

        response
            .json::<ChatResponse>()
            .await?
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| DeepSeekError::Api("LLM response contained no choices".into()))
    }

    pub fn parse_tool_calls(&self, llm_output: &str) -> Vec<String> {
        // Placeholder for parsing tool calls from the LLM output
        // In a real implementation, this would analyze the LLM output and extract any tool calls
        if llm_output.contains("tool_call") {
            vec!["tool_call_example".to_string()]
        } else {
            Vec::new()
        }
    }

    pub fn execute_action(&self, action: String) -> String {
        // Placeholder for executing the action
        // In a real implementation, this would perform the action and return the result
        format!("Executed action: {}", action)
    }

    pub fn chat(&self) -> ChatBuilder<'_> {
        ChatBuilder::new(self)
    }
}
