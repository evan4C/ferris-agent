use serde_json::Value;
use std::borrow::Cow;

/// Cow 既保留了内置 Tool 的零成本性能，又为未来动态加载 Tool、MCP Server 和插件系统留下了扩展空间
#[derive(Clone)]
pub struct ToolMetadata {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub schema: Value,
}