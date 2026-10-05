use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("API key is not configured; set the {0} environment variable")]
    NotFound(String),
}

pub fn get_api_key(provider: &str) -> Result<String, CredentialError> {
    let env_name = api_key_env_var(provider);
    std::env::var(&env_name)
        .ok()
        .filter(|key| !key.trim().is_empty())
        .ok_or(CredentialError::NotFound(env_name))
}

fn api_key_env_var(provider: &str) -> String {
    let provider = provider
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("FERRIS_AGENT_{provider}_API_KEY")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_provider_environment_variable_name() {
        assert_eq!(api_key_env_var("deepseek"), "FERRIS_AGENT_DEEPSEEK_API_KEY");
    }

    #[test]
    fn missing_key_reports_variable_name() {
        let error = get_api_key("ferris-test-unset").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("FERRIS_AGENT_FERRIS_TEST_UNSET_API_KEY")
        );
    }
}
