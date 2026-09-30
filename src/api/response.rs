use crate::api::request::Model;
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

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// A single chunk of a server-sent-events streaming response.
#[derive(Debug, Deserialize)]
pub struct ChatStreamChunk {
    pub choices: Vec<ChatStreamChoice>,
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

impl ChatUsage {
    /// Estimated cost in USD for this usage, based on the given model's per-token pricing.
    pub fn cost_usd(&self, model: &Model) -> f64 {
        let pricing = model.pricing();
        let input_cost = self.prompt_tokens as f64 / 1_000_000.0 * pricing.input_per_million_usd;
        let output_cost =
            self.completion_tokens as f64 / 1_000_000.0 * pricing.output_per_million_usd;
        input_cost + output_cost
    }
}

impl std::ops::AddAssign for ChatUsage {
    fn add_assign(&mut self, rhs: Self) {
        self.prompt_tokens += rhs.prompt_tokens;
        self.completion_tokens += rhs.completion_tokens;
        self.total_tokens += rhs.total_tokens;
    }
}

/// Approximate per-million-token pricing in USD, used to estimate request cost.
pub struct ModelPricing {
    pub input_per_million_usd: f64,
    pub output_per_million_usd: f64,
}

impl Model {
    pub fn pricing(&self) -> ModelPricing {
        match self {
            Model::Flash => ModelPricing {
                input_per_million_usd: 0.14,
                output_per_million_usd: 0.28,
            },
            Model::Pro => ModelPricing {
                input_per_million_usd: 0.55,
                output_per_million_usd: 2.19,
            },
        }
    }
}
