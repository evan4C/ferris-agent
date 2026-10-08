use crate::api::cost::ChatUsage;
use crate::tool::ToolCall;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub finish_reason: Option<String>,
    pub message: ChatMessage,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,

    #[serde(default)]
    pub reasoning_content: Option<String>,

    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
}

/// A single chunk of a server-sent-events streaming response.
#[derive(Debug, Deserialize)]
pub struct ChatStreamChunk {
    pub choices: Vec<ChatStreamChoice>,
    #[serde(default)]
    pub usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
pub struct ChatStreamChoice {
    pub delta: ChatDelta,
}

/// Incremental piece of an assistant message delivered while streaming.
#[derive(Debug, Default, Deserialize)]
pub struct ChatDelta {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ChatToolCallDelta>>,
}

#[derive(Debug, Deserialize)]
pub struct ChatToolCallDelta {
    pub index: usize,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub function: Option<ChatToolCallFunctionDelta>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ChatToolCallFunctionDelta {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arguments: Option<String>,
}
