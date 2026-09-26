use serde::{Deserialize, Serialize, ser::SerializeMap};
use crate::DeepSeekClient;
use crate::config;

// region: Request structs
#[derive(Debug, Serialize)]
pub enum Model {
    #[serde(rename = "deepseek-flash")]
    Flash,
    #[serde(rename = "deepseek-v4-pro")]
    Pro,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Serialize)]
pub struct Message {
    role: Role,
    content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }

    pub fn tool(content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
        }
    }
}

#[derive(Debug)]
pub enum Thinking {
    Enabled,
    Disabled,
}

impl Serialize for Thinking {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Thinking::Enabled => map.serialize_entry("type", "enabled")?,
            Thinking::Disabled => map.serialize_entry("type", "disabled")?,
        }
        map.end()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingEffort {
    None,
    Low,
    High,
    Max,
}

#[derive(Debug)]
pub enum ResponseFormat {
    JsonObject,
    Text,
}

impl Serialize for ResponseFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;

        match self {
            ResponseFormat::JsonObject => map.serialize_entry("type", "json_object")?,
            ResponseFormat::Text => map.serialize_entry("type", "text")?,
        }
        map.end()
    }
}
        

#[derive(Debug, Serialize)]
pub struct StreamOptions {
    include_usage: bool,
}

#[derive(Debug, Serialize)]
pub struct ToolFunc {
    description: Option<String>,
    name: String,
    strict: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function,
}

#[derive(Debug, Serialize)]
pub struct Tools {
    #[serde(rename = "type")]
    kind: ToolType,
    function: ToolFunc,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SimpleTC {
    None,
    Auto,
    Required,
}

#[derive(Debug, Serialize)]
pub struct NamedTC {
    #[serde(rename = "type")]
    kind: ToolType,
    name: String,
}

#[derive(Debug, Serialize)]
pub enum ToolChoice {
    SimpleToolChoice(SimpleTC),
    NamedToolChoice(NamedTC),
}

#[derive(Debug, Serialize)]
pub struct ChatBuilder<'a> {
    // Not part of the API payload; drives `create()`.
    #[serde(skip)]
    client: &'a DeepSeekClient,

    pub messages: Vec<Message>,
    pub model: Model,

    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<Thinking>,
    reasoning_effort: ThinkingEffort,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

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

impl<'a> ChatBuilder<'a> {
    pub fn new(client: &'a DeepSeekClient) -> Self {
        Self {
            client,
            messages: Vec::new(),
            model: Model::Flash,
            thinking: None,
            reasoning_effort: ThinkingEffort::None,
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
        }.system(config::SYSTEM_PROMPT)
    }

    pub fn model(mut self, model: Model) -> Self {
        self.model = model;
        self
    }

    pub fn system(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::system(content));
        self
    }
    pub fn user(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::user(content));
        self
    }
    pub fn assistant(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::assistant(content));
        self
    }
    pub fn tool(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::tool(content));
        self
    }

    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn stream(mut self, enabled: bool) -> Self {
        self.stream = Some(enabled);
        self
    }

    pub fn json_output(mut self) -> Self {
        self.response_format = Some(ResponseFormat::JsonObject);
        self
    }

    pub fn text_output(mut self) -> Self {
        self.response_format = Some(ResponseFormat::Text);
        self
    }

    pub async fn create(self) -> Result<String, crate::error::DeepSeekError> {
        self.client.query_llm(&self).await
    }
}

// endregion: Request structs

// region: Response structs
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
// endregion: Response structs