use crate::tool::{Tool, ToolContext, ToolMetadata, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::borrow::Cow;

pub struct WriteFileTool;

#[derive(serde::Deserialize)]
struct WriteFileArgs {
    path: String,
    content: String,
}

#[async_trait]
impl Tool for WriteFileTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: Cow::Borrowed("write_file"),
            description: Cow::Borrowed("Write a text file."),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn execute(&self, ctx: &ToolContext, args: Value) -> Result<ToolResult> {
        let args: WriteFileArgs = serde_json::from_value(args)?;
        let path = ctx.working_dir.join(args.path);
        tokio::fs::write(path, args.content).await?;

        Ok(ToolResult {
            content: "".to_string(),
            is_error: false,
            metadata: None,
        })
    }
}
