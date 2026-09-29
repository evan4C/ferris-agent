use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::borrow::Cow;
use tokio::process::Command;

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
        let output = Command::new("git")
            .args(["status", "--short", "--branch"])
            .current_dir(&ctx.working_dir)
            .output()
            .await?;

        let mut content = String::from_utf8_lossy(&output.stdout).into_owned();
        content.push_str(&String::from_utf8_lossy(&output.stderr));

        // TODO: split content into stdout and stderr separately
        Ok(ToolResult {
            content,
            is_error: !output.status.success(),
            metadata: Some(serde_json::json!({ "exit_code": output.status.code() })),
        })
    }
}
