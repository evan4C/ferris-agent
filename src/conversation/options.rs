use crate::api::Model;

#[derive(Clone, Debug)]
pub struct ChatOptions {
    pub model: Model,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            model: Model::Flash,
            max_tokens: None,
            stream: false,
        }
    }
}
