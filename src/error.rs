use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeepSeekError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error: {0}")]
    Api(String),
}