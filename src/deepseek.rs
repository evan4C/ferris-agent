use std::env;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct DSAgent {
    base_url: String,
    headers: HeaderMap,
    pub request_body: RequestBody,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestBody {
    model: String,
    messages: Vec<Message>,
    thinking: HashMap<String, String>,
    reasoning_effort: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: String,
}

impl DSAgent {
    pub fn new() -> Self {
        let key = env::var("DEEP_SEEK_API_KEY").expect("Environment variable 'DEEP_SEEK_API_KEY' is not set");
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", key)).unwrap());

        let mut thinking = HashMap::new();
        // enable thinking mode
        thinking.insert("type".to_string(), "enabled".to_string());
        let reasoning_effort = String::from("low");

        let request_body = RequestBody::new(
            String::from("deepseek-flash"),
            Vec::new(),
            thinking,
            reasoning_effort,
            false,
        );

        DSAgent {
            base_url: String::from("https://api.deepseek.com/chat/completions"),
            headers,
            request_body,
        }
    }

    pub async fn query_llm(&self) -> anyhow::Result<String> {

        let client = reqwest::Client::new();
        let request_body = serde_json::json!({
            "model": self.request_body.model,
            "messages": self.request_body.messages,
            "thinking": self.request_body.thinking,
            "reasoning_effort": self.request_body.reasoning_effort,
            "stream": self.request_body.stream,
        });

        let response = client.post(&self.base_url)
            .headers(self.headers.clone())
            .json(&request_body)
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
    pub fn new(model: String, messages: Vec<Message>, thinking: HashMap<String, String>, reasoning_effort: String, stream: bool) -> Self {
        RequestBody {
            model,
            messages,
            thinking,
            reasoning_effort,
            stream,
        }
    }

    pub fn add_system_prompt(&mut self, system_prompt: String) {
        self.messages.insert(0, Message {
            role: String::from("system"),
            content: system_prompt,
        });
    }

    pub fn add_user_message(&mut self, user_message: String) {
        self.messages.push(Message {
            role: String::from("user"),
            content: user_message,
        });
    }

    pub fn add_assistant_message(&mut self, assistant_message: String) {
        self.messages.push(Message {
            role: String::from("assistant"),
            content: assistant_message,
        });
    }
    
    pub fn add_toolcall_message(&mut self, toolcall_message: String) {
        self.messages.push(Message {
            role: String::from("tool"),
            content: toolcall_message,
        });
    }
}
    