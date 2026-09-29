use crate::api::DeepSeekError;
use crate::api::request::ChatCompletionRequest;
use crate::api::response::{ChatMessage, ChatResponse};
use crate::config::SETTINGS;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

pub struct DeepSeekClient {
    client: reqwest::Client,
}

impl DeepSeekClient {
    pub fn new() -> Self {
        DeepSeekClient {
            client: reqwest::Client::new(),
        }
    }

    pub async fn http_request(
        &self,
        request_body: &ChatCompletionRequest<'_>,
    ) -> Result<ChatMessage, DeepSeekError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", SETTINGS.deepseek.api_key)).unwrap(),
        );

        let request_body = serde_json::to_value(request_body)?;

        let response = self
            .client
            .post(&SETTINGS.deepseek.base_url)
            .headers(headers)
            .json(&request_body)
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

        let mut response = response.json::<ChatResponse>().await?;
        if response.choices.is_empty() {
            return Err(DeepSeekError::Api(
                "LLM response contained no choices".into(),
            ));
        }
        Ok(response.choices.remove(0).message)
    }

    pub fn chat(&self) -> ChatCompletionRequest<'_> {
        ChatCompletionRequest::new(self)
    }
}
