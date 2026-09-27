use std::time::Duration;

pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub fn default_timeout() -> Duration {
    Duration::from_secs(DEFAULT_TIMEOUT_SECS)
}

pub const SYSTEM_PROMPT: &str = "You are a helpful assistant.";
