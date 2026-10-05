use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unable to determine the user configuration directory")]
    DirectoryUnavailable,
    #[error("failed to read configuration: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse configuration: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize configuration: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("invalid configuration value for {0}")]
    InvalidValue(String),
}

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

impl AppConfig {
    pub fn config_dir() -> Result<PathBuf, ConfigError> {
        ProjectDirs::from("", "", "ferris-agent")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .ok_or(ConfigError::DirectoryUnavailable)
    }

    pub fn config_path() -> Result<PathBuf, ConfigError> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from(&Self::config_path()?)
    }

    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        match fs::read_to_string(path) {
            Ok(contents) => {
                let config: Self = toml::from_str(&contents)?;
                config.validate()?;
                Ok(config)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.deepseek.model.trim().is_empty() {
            return Err(ConfigError::InvalidValue("deepseek.model".into()));
        }
        if !(self.deepseek.base_url.starts_with("https://")
            || self.deepseek.base_url.starts_with("http://"))
        {
            return Err(ConfigError::InvalidValue("deepseek.base_url".into()));
        }
        if self.agent.max_iterations == 0 {
            return Err(ConfigError::InvalidValue("agent.max_iterations".into()));
        }
        Ok(())
    }

    pub fn initialize() -> Result<PathBuf, ConfigError> {
        let path = Self::config_path()?;
        fs::create_dir_all(path.parent().ok_or(ConfigError::DirectoryUnavailable)?)?;
        let contents = toml::to_string_pretty(&Self::default())?;
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                use std::io::Write;
                file.write_all(contents.as_bytes())?;
                Ok(path)
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(path),
            Err(error) => Err(error.into()),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_available_without_a_config_file() {
        assert_eq!(AppConfig::default().deepseek.model, "deepseek-chat");
        assert_eq!(AppConfig::default().agent.max_iterations, 10);
    }

    #[test]
    fn parses_partial_toml_with_defaults() {
        let config: AppConfig = toml::from_str("[deepseek]\nmodel = 'custom'\n").unwrap();
        assert_eq!(config.deepseek.model, "custom");
        assert_eq!(
            config.deepseek.base_url,
            "https://api.deepseek.com/chat/completions"
        );
        assert_eq!(config.agent.max_iterations, 10);
    }

    #[test]
    fn reports_invalid_toml() {
        assert!(toml::from_str::<AppConfig>("[deepseek\n").is_err());
    }

    #[test]
    fn rejects_secrets_and_unknown_fields_in_toml() {
        assert!(toml::from_str::<AppConfig>("[deepseek]\napi_key = 'secret'\n").is_err());
    }

    #[test]
    fn validates_required_values() {
        let mut config = AppConfig::default();
        config.deepseek.model = " ".into();
        assert!(config.validate().is_err());

        let mut config = AppConfig::default();
        config.deepseek.base_url = "not-a-url".into();
        assert!(config.validate().is_err());

        let mut config = AppConfig::default();
        config.agent.max_iterations = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn missing_file_uses_defaults() {
        let path = std::env::temp_dir().join(format!("ferris-agent-{}.toml", std::process::id()));
        let _ = fs::remove_file(&path);
        assert_eq!(
            AppConfig::load_from(&path).unwrap().agent.max_iterations,
            10
        );
    }

    #[test]
    fn resolves_platform_configuration_directory() {
        assert!(AppConfig::config_path().unwrap().ends_with("config.toml"));
    }
}
