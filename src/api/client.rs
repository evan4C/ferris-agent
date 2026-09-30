use crate::api::DeepSeekError;
use crate::api::request::ChatCompletionRequest;
use crate::api::response::{ChatMessage, ChatResponse};
use crate::config::DeepSeekConfig;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

pub struct DeepSeekClient {
    client: reqwest::Client,
    base_url: String,
    authorization: HeaderValue,
}

impl DeepSeekClient {
    pub fn new(config: DeepSeekConfig, api_key: String) -> Result<Self, DeepSeekError> {
        let mut authorization = HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|_| DeepSeekError::Api("invalid API credential".into()))?;
        authorization.set_sensitive(true);
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: config.base_url,
            authorization,
        })
    }

    pub async fn http_request(
        &self,
        request_body: &ChatCompletionRequest<'_>,
    ) -> Result<ChatMessage, DeepSeekError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, self.authorization.clone());

        let request_body = serde_json::to_value(request_body)?;

        let response = self
            .client
            .post(&self.base_url)
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
