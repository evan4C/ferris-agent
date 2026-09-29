use serde_json::Value;

pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
    pub metadata: Option<Value>,
}



