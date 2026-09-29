pub struct ReadFileTool;
use crate::tool::{Tool, ToolContext, ToolMetadata, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::borrow::Cow;

#[derive(serde::Deserialize)]
struct ReadFileArgs {
    path: String,
}

#[async_trait]
impl Tool for ReadFileTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: Cow::Borrowed("read_file"),
            description: Cow::Borrowed("Read a text file."),
            schema: serde_json::json!({
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<ToolResult> {
        let args: ReadFileArgs = serde_json::from_value(args)?;
        let path = ctx.working_dir.join(args.path);
        let text = tokio::fs::read_to_string(path).await?;

        Ok(ToolResult {
            content: text,
            is_error: false,
            metadata: None,
        })
    }
}
