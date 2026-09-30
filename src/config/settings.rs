use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    pub deepseek: DeepSeekConfig,
    pub agent: AgentConfig,
    pub tools: ToolConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DeepSeekConfig {
    pub model: String,
    pub base_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgentConfig {
    pub max_iterations: u8,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ToolConfig {
    pub workspace: Option<PathBuf>,
}

impl Default for DeepSeekConfig {
    fn default() -> Self {
        Self {
            model: "deepseek-chat".into(),
            base_url: "https://api.deepseek.com/chat/completions".into(),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self { max_iterations: 10 }
    }
}
