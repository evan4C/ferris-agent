use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::borrow::Cow;

use super::commands::run_git;
use crate::tool::{Tool, ToolContext, ToolMetadata, ToolResult};

pub struct GitStatusTool;

#[async_trait]
impl Tool for GitStatusTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: Cow::Borrowed("git_status"),
            description: Cow::Borrowed("Show the workspace git status."),
            schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, ctx: &ToolContext, _args: Value) -> Result<ToolResult> {
        run_git(ctx, &["status".into(), "--short".into(), "--branch".into()]).await
    }
}
