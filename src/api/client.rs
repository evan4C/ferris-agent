use crate::api::Thinking;
use crate::api::error::DeepSeekError;
use crate::api::request::ChatCompletionRequest;
use crate::api::response::{ChatMessage, ChatResponse, ChatStreamChunk, ChatUsage};
use crate::config::DeepSeekConfig;
use crate::tool::ToolCall;
use futures_util::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use std::collections::BTreeMap;
use std::io::Write;

/// A single model turn along with the token usage billed for it.
pub struct ApiTurn {
    pub message: ChatMessage,
    pub usage: Option<ChatUsage>,
}

pub struct DeepSeekClient {
    client: reqwest::Client,
    base_url: String,
    authorization: HeaderValue,
}

/// Accumulates the fragments of a single tool call as they arrive across stream chunks.
#[derive(Default)]
struct PartialToolCall {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
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
    ) -> Result<ApiTurn, DeepSeekError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, self.authorization.clone());

        let is_stream = request_body.stream.unwrap_or(false);
        let is_thinking = if matches!(
            request_body.thinking.unwrap_or(Thinking::Disabled),
            Thinking::Enabled
        ) {
            true
        } else {
            false
        };
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

        if is_stream {
            Self::handle_stream_response(response, is_thinking).await
        } else {
            let mut response = response.json::<ChatResponse>().await?;
            if response.choices.is_empty() {
                return Err(DeepSeekError::Api(
                    "LLM response contained no choices".into(),
                ));
            }
            let message = response.choices.remove(0).message;
            if is_thinking && let Some(reasoning) = &message.reasoning_content {
                eprintln!("Thinking: {}", reasoning);
            }
            Ok(ApiTurn { message, usage: response.usage })
        }
        }
    }

    /// Reads the SSE stream chunk by chunk, printing reasoning as it arrives and
    /// accumulating content/tool calls into a single `ChatMessage`.
    async fn handle_stream_response(
        response: reqwest::Response,
        is_thinking: bool,
    ) -> Result<ChatMessage, DeepSeekError> {
        let mut byte_stream = response.bytes_stream();
        let mut buffer = Vec::new();
        let mut role = String::from("assistant");
        let mut content = String::new();
        let mut reasoning_content = String::new();
        let mut reasoning_started = false;
        let mut tool_calls: BTreeMap<usize, PartialToolCall> = BTreeMap::new();

        while let Some(chunk) = byte_stream.next().await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);

            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line = String::from_utf8_lossy(&buffer[..pos])
                    .trim_end_matches('\r')
                    .to_string();
                buffer.drain(..=pos);

                let Some(data) = line.strip_prefix("data: ") else {
                    continue;
                };
                if data == "[DONE]" {
                    continue;
                }

                let chunk: ChatStreamChunk = serde_json::from_str(data)?;

                for choice in chunk.choices {
                    let delta = choice.delta;
                    if let Some(delta_role) = delta.role {
                        role = delta_role;
                    }
                    if let Some(reasoning) = delta.reasoning_content {
                        if is_thinking {
                            if !reasoning_started {
                                eprint!("Thinking: ");
                                reasoning_started = true;
                            }
                            eprint!("{}", reasoning);
                            let _ = std::io::stderr().flush();
                        }
                        reasoning_content.push_str(&reasoning);
                    }
                    if let Some(delta_content) = delta.content {
                        print!("{delta_content}");
                        let _ = std::io::stdout().flush();
                        content.push_str(&delta_content);
                    }
                    if let Some(delta_tool_calls) = delta.tool_calls {
                        for delta_call in delta_tool_calls {
                            let entry = tool_calls.entry(delta_call.index).or_default();
                            if let Some(id) = delta_call.id {
                                entry.id = Some(id);
                            }
                            if let Some(function) = delta_call.function {
                                if let Some(name) = function.name {
                                    entry.name.get_or_insert_with(String::new).push_str(&name);
                                }
                                if let Some(arguments) = function.arguments {
                                    entry.arguments.push_str(&arguments);
                                }
                            }
                        }
                    }
                }
            }
        }

        if is_thinking && reasoning_started {
            eprintln!();
        }

        let tool_calls = tool_calls
            .into_values()
            .map(|call| {
                ToolCall::new(
                    call.id.unwrap_or_default(),
                    call.name.unwrap_or_default(),
                    call.arguments,
                )
            })
            .collect();

        Ok(ChatMessage {
            role,
            content: if content.is_empty() {
                None
            } else {
                Some(content)
            },
            reasoning_content: if reasoning_content.is_empty() {
                None
            } else {
                Some(reasoning_content)
            },
            tool_calls,
        })
    }

    pub fn chat(&self) -> ChatCompletionRequest<'_> {
        ChatCompletionRequest::new(self)
    }
}
