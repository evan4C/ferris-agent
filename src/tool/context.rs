use std::collections::HashMap;
use std::path::PathBuf;

pub struct ToolContext {
    pub working_dir: PathBuf,
    pub env: HashMap<String, String>,
}

impl ToolContext {
    pub fn new(working_dir: PathBuf, env: HashMap<String, String>) -> Self {
        Self { working_dir, env }
    }
}
