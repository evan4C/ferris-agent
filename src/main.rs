use anyhow::Result;
use ferris_agent::{Conversation, DeepSeekClient};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Arc::new(DeepSeekClient::new());
    let mut conversation = Conversation::new(agent);

    let reply = conversation
        .send("write a new file called hello-world.txt and come up with a computer science joke and write into it")
        .await?;
    println!("LLM output: {}", reply);
    Ok(())
}
