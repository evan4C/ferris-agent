use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub deepseek: DeepSeekConfig,
}

#[derive(Debug, Deserialize)]
pub struct DeepSeekConfig {
    pub api_key: String,
    pub base_url: String,
    pub max_retries: u8,
}
