use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod context;
pub mod fs;
pub mod git;
pub mod metadata;
pub mod registry;
pub mod result;
pub mod shell;

pub use context::*;
pub use fs::*;
pub use metadata::*;
pub use result::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type", default = "tool_call_type")]
    kind: String,
    pub function: ToolCallFunction,
}

impl ToolCall {
    pub fn new(id: String, function: ToolCallFunction) -> Self {
        Self {
            id,
            kind: tool_call_type(),
            function,
        }
    }
}

fn tool_call_type() -> String {
    "function".to_string()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn metadata(&self) -> ToolMetadata;

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<ToolResult>;
}
