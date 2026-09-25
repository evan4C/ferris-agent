use serde::{Deserialize, Serialize};

// Request structs
#[derive(Debug, Serialize, Deserialize)]
pub enum Model {
    #[serde(rename = "deepseek-flash")]
    Flash,
    #[serde(rename = "deepseek-v4-pro")]
    Pro,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Enable {
    Enabled,
    Disabled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thinking {
    #[serde(rename = "type")]
    kind: Enable,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingEffort {
    None,
    Low,
    High,
    Max,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormatKind {
    #[serde(rename = "json_object")]
    Json,
    Text,
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
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function,
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
    pub content: String,

    #[serde(default)]
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
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
    