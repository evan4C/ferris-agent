use std::env;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde::{Deserialize, Serialize};

use crate::enums::{Model, Role, ToolType, ThinkingEffort, ResponseFormatKind, Enable};

// Request structs

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thinking {
    #[serde(rename = "type")]
    kind: Enable,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    kind: ResponseFormatKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamOptions {
    include_usage: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolFunc {
    description: Option<String>,
    name: String,
    strict: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tools {
    #[serde(rename = "type")]
    kind: ToolType,
    function: ToolFunc,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SimpleTC {
    None,
    Auto,
    Required,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamedTC {
    #[serde(rename = "type")]
    kind: ToolType,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ToolChoice {
    SimpleToolChoice(SimpleTC),
    NamedToolChoice(NamedTC),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
    messages: Vec<Message>,
    model: Model,
    thinking: Option<Thinking>,
    reasoning_effort: ThinkingEffort,

    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tools>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_logprobs: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
}

// Response structs

#[derive(Debug, Deserialize)]
struct ChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<ChatChoice>,
    usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    index: u32,
    finish_reason: Option<String>,
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,

    #[serde(default)]
    reasoning_content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct DSAgent {
    base_url: String,
    headers: HeaderMap,
    pub request_body: RequestBody,
    client: reqwest::Client,
}

impl DSAgent {
    pub fn new() -> Self {
        let key = env::var("DEEP_SEEK_API_KEY").expect("Environment variable 'DEEP_SEEK_API_KEY' is not set");
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", key)).unwrap());

        let request_body = RequestBody::new(
            Model::Flash,
            Vec::new(),
        );

        DSAgent {
            base_url: String::from("https://api.deepseek.com/chat/completions"),
            headers,
            request_body,
            client: reqwest::Client::new(),
        }
    }

    pub async fn query_llm(&self) -> anyhow::Result<String> {
        let response = self.client.post(&self.base_url)
            .headers(self.headers.clone())
            .json(&self.request_body)
            .send()
            .await?
            .error_for_status()?
            .json::<ChatResponse>()
            .await?;

        response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("LLM response contained no choices"))
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
}

impl RequestBody {
    pub fn new(model: Model, messages: Vec<Message>) -> Self {
        RequestBody {
            model,
            messages,
            thinking: Some(Thinking { kind: Enable::Disabled }),
            reasoning_effort: ThinkingEffort::Low,
            max_tokens: None,
            response_format: None,
            stream: None,
            stream_options: None,
            temperature: None,
            top_p: None,
            tools: None,
            tool_choice: None,
            logprobs: None,
            top_logprobs: None,
            user_id: None,
        }
    }

    pub fn add_system_prompt(&mut self, system_prompt: String) {
        self.messages.insert(0, Message {
            role: Role::System,
            content: system_prompt,
        });
    }

    pub fn add_user_message(&mut self, user_message: String) {
        self.messages.push(Message {
            role: Role::User,
            content: user_message,
        });
    }

    pub fn add_assistant_message(&mut self, assistant_message: String) {
        self.messages.push(Message {
            role: Role::Assistant,
            content: assistant_message,
        });
    }
    
    pub fn add_toolcall_message(&mut self, toolcall_message: String) {
        self.messages.push(Message {
            role: Role::Tool,
            content: toolcall_message,
        });
    }
}
    