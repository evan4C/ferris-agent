use std::sync::Arc;
use crate::DeepSeekClient;
use crate::ToolRegistry;
use crate::Message;

pub struct Conversation {
    client: Arc<DeepSeekClient>,
    registry: Arc<ToolRegistry>,
    messages: Vec<Message>,
}

impl Conversation {
    pub fn new(client: Arc<DeepSeekClient>, registry: Arc<ToolRegistry>) -> Self {
        Self {
            client,
            registry,
            messages: Vec::new(),
        }
    }
}