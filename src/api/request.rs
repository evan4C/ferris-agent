use crate::api::client::DeepSeekClient;
use crate::api::error::DeepSeekError;
use crate::api::message::Message;
use crate::config;
use crate::tool::registry::ToolDefinition;
use serde::{Serialize, ser::SerializeMap};

#[derive(Clone, Debug, Serialize)]
pub enum Model {
    #[serde(rename = "deepseek-flash")]
    Flash,
    #[serde(rename = "deepseek-v4-pro")]
    Pro,
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
#[serde(rename_all = "lowercase")]
pub enum ChatCompletionToolChoice {
    None,
    Auto,
    Required,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionNamedToolChoice {
    #[serde(rename = "type")]
    kind: String,
    function: ChatCompletionNamedToolChoiceFunction,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionNamedToolChoiceFunction {
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Choice(ChatCompletionToolChoice),
    Named(ChatCompletionNamedToolChoice),
}

#[derive(Serialize)]
pub struct ChatCompletionRequest<'a> {
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
    stream_options: Option<StreamOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_logprobs: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
}

impl<'a> ChatCompletionRequest<'a> {
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
        }
        .system(config::constants::SYSTEM_PROMPT)
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

    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
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

    pub fn tools(mut self, definitions: Vec<ToolDefinition>) -> Self {
        if !definitions.is_empty() {
            self.tools = Some(definitions);
            self.tool_choice = Some(ToolChoice::Choice(ChatCompletionToolChoice::Auto));
        }
        self
    }

    pub async fn create(self) -> Result<String, DeepSeekError> {
        let message = self.client.http_request(&self).await?;
        Ok(message.content.unwrap_or_default())
    }
}
