use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use crate::tool::*;

struct RegisteredTool {
    metadata: ToolMetadata,
    handler: Arc<dyn Tool>,
}

#[derive(Debug, Serialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    kind: &'static str,
    pub function: FunctionDefinition,
}

#[derive(Debug, Serialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

pub struct ToolRegistry {
    tools: HashMap<String, RegisteredTool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn builder() -> ToolRegistryBuilder {
        ToolRegistryBuilder {
            registry: ToolRegistry::new(),
        }
    }

    pub fn register<T>(&mut self, tool: T) -> &mut Self
    where
        T: Tool + 'static,
    {
        let meta = tool.metadata();

        self.tools.insert(
            meta.name.to_string(),
            RegisteredTool {
                metadata: meta,
                handler: Arc::new(tool),
            },
        );

        self
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|t| ToolDefinition {
                kind: "function",
                function: FunctionDefinition {
                    name: t.metadata.name.to_string(),
                    description: t.metadata.description.to_string(),
                    parameters: t.metadata.schema.clone(),
                },
            })
            .collect()
    }

    pub async fn dispatch(&self, ctx: &ToolContext, call: &ToolCall) -> Result<ToolResult> {
        let tool = self
            .tools
            .get(&call.function.name)
            .ok_or_else(|| anyhow::anyhow!("Unknown tool: {}", call.function.name))?;

        let args: Value = serde_json::from_str(&call.function.arguments)?;

        tool.handler.execute(ctx, args).await
    }

    pub async fn dispatch_many(
        &self,
        ctx: &ToolContext,
        calls: &[ToolCall],
    ) -> Result<Vec<(String, ToolResult)>> {
        let mut results = Vec::with_capacity(calls.len());
        for call in calls {
            let result = match self.dispatch(ctx, call).await {
                Ok(result) => result,
                Err(error) => ToolResult {
                    content: error.to_string(),
                    is_error: true,
                    metadata: None,
                },
            };
            results.push((call.id.clone(), result));
        }
        Ok(results)
    }
}

pub struct ToolRegistryBuilder {
    registry: ToolRegistry,
}

impl ToolRegistryBuilder {
    pub fn filesystem(mut self) -> Self {
        self.registry.register(fs::read::ReadFileTool);
        self.registry.register(fs::write::WriteFileTool);
        self
    }

    pub fn shell(mut self) -> Self {
        self.registry.register(shell::bash::BashTool);
        self
    }

    pub fn git(mut self) -> Self {
        self.registry.register(git::status::GitStatusTool);
        self
    }

    pub fn build(self) -> ToolRegistry {
        self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn test_directory() -> std::path::PathBuf {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ferris-agent-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn definitions_use_function_tool_format() {
        let registry = ToolRegistry::builder().filesystem().shell().git().build();
        let definitions = serde_json::to_value(registry.definitions()).unwrap();

        assert_eq!(definitions[0]["type"], "function");
        let names: Vec<_> = definitions
            .as_array()
            .unwrap()
            .iter()
            .map(|definition| definition["function"]["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"read_file"));
        assert!(names.contains(&"write_file"));
        assert!(names.contains(&"bash"));
        assert!(names.contains(&"git_status"));
    }

    #[tokio::test]
    async fn dispatches_file_tool_relative_to_working_directory() {
        let directory = test_directory();
        std::fs::write(directory.join("sample.txt"), "tool result").unwrap();

        let call = serde_json::from_value(json!({
            "id": "call-1",
            "type": "function",
            "function": {
                "name": "read_file",
                "arguments": "{\"path\":\"sample.txt\"}"
            }
        }))
        .unwrap();
        let context = ToolContext::new(directory.clone(), HashMap::new());
        let result = ToolRegistry::builder()
            .filesystem()
            .build()
            .dispatch(&context, &call)
            .await
            .unwrap();

        assert_eq!(result.content, "tool result");
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[tokio::test]
    async fn returns_tool_errors_for_missing_files() {
        let directory = test_directory();

        let call = serde_json::from_value(json!({
            "id": "call-missing",
            "type": "function",
            "function": {
                "name": "read_file",
                "arguments": "{\"path\":\"missing.txt\"}"
            }
        }))
        .unwrap();
        let context = ToolContext::new(directory.clone(), HashMap::new());
        let results = ToolRegistry::builder()
            .filesystem()
            .build()
            .dispatch_many(&context, &[call])
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "call-missing");
        assert!(results[0].1.is_error);
        assert!(!results[0].1.content.is_empty());

        std::fs::remove_dir_all(directory).unwrap();
    }
}
