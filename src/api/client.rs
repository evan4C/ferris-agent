use anyhow::Result;
use crate::tool::tool_call_type;
use crate::api::request::ChatCompletionRequest;
use crate::api::response::{ChatCompletionChunk, ChatMessage, ChatResponse};
use crate::config::SETTINGS;
use crate::tool::{ToolCall, ToolCallFunction};
use futures_util::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use std::io::Write;

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

        let is_stream = request_body.stream.unwrap_or(false);
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

        if is_stream {
            return self.read_stream(response).await;
        }

        let mut response = response.json::<ChatResponse>().await?;
        if response.choices.is_empty() {
            return Err(DeepSeekError::Api(
                "LLM response contained no choices".into(),
            ));
        }
        Ok(response.choices.remove(0).message)
    }

    /// Consumes a text/event-stream response, printing content deltas as they
    /// arrive and reassembling the full message once the stream ends.
    async fn read_stream(
        &self,
        response: reqwest::Response,
    ) -> Result<ChatMessage, DeepSeekError> {
        let mut byte_stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut role = String::from("assistant");
        let mut content = String::new();
        let mut tool_calls: Vec<(String, String, String)> = Vec::new();
        let mut stdout = std::io::stdout();

        while let Some(chunk) = byte_stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim_end_matches('\r').to_string();
                buffer.drain(..=pos);

                let Some(data) = line.strip_prefix("data: ") else {
                    continue;
                };
                if data.is_empty() || data == "[DONE]" {
                    continue;
                }

                let chunk: ChatCompletionChunk = match serde_json::from_str(data) {
                    Ok(chunk) => chunk,
                    Err(_) => continue,
                };

                for choice in chunk.choices {
                    if let Some(delta_role) = choice.delta.role {
                        role = delta_role;
                    }
                    if let Some(text) = choice.delta.content {
                        print!("{text}");
                        let _ = stdout.flush();
                        content.push_str(&text);
                    }
                    if let Some(deltas) = choice.delta.tool_calls {
                        for delta in deltas {
                            while tool_calls.len() <= delta.index {
                                tool_calls.push((String::new(), String::new(), String::new()));
                            }
                            let entry = &mut tool_calls[delta.index];
                            if let Some(id) = delta.id {
                                entry.0 = id;
                            }
                            if let Some(function) = delta.function {
                                if let Some(name) = function.name {
                                    entry.1.push_str(&name);
                                }
                                if let Some(arguments) = function.arguments {
                                    entry.2.push_str(&arguments);
                                }
                            }
                        }
                    }
                }
            }
        }

        if !content.is_empty() {
            println!();
        }

        Ok(ChatMessage {
            role,
            content: if content.is_empty() {
                None
            } else {
                Some(content)
            },
            reasoning_content: None,
            tool_calls: tool_calls
                .into_iter()
                .map(|(id, name, arguments)| ToolCall::new(id, ToolCallFunction { name, arguments }))
                .collect(),
        })
    }

    pub fn chat(&self) -> ChatCompletionRequest<'_> {
        ChatCompletionRequest::new(self)
    }
}

