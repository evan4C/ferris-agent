use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::borrow::Cow;
use tokio::process::Command;

use crate::tool::{Tool, ToolContext, ToolMetadata, ToolResult};

pub struct BashTool;

#[derive(Deserialize)]
struct BashArgs {
    command: String,
}

#[async_trait]
impl Tool for BashTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: Cow::Borrowed("bash"),
            description: Cow::Borrowed("Run a shell command in the workspace."),
            schema: serde_json::json!({
                "type": "object",
                "properties": { "command": { "type": "string" } },
                "required": ["command"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<ToolResult> {
        let args: BashArgs = serde_json::from_value(args)?;
        let output = Command::new("sh")
            .arg("-c")
            .arg(args.command)
            .current_dir(&ctx.working_dir)
            .envs(&ctx.env)
            .output()
            .await?;

        let mut content = String::from_utf8_lossy(&output.stdout).into_owned();
        content.push_str(&String::from_utf8_lossy(&output.stderr));

        Ok(ToolResult {
            content,
            is_error: !output.status.success(),
            metadata: Some(serde_json::json!({ "exit_code": output.status.code() })),
        })
    }
}
