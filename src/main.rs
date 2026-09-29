use anyhow::Result;
use ferris_agent::DeepSeekClient;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = DeepSeekClient::new();
    let user_prompt = "write a new file called hello.txt and come up with a joke and write into it";
    let request = agent.chat().user(user_prompt);
    let response = agent.query_llm(&request).await?;
    println!("LLM output: {}", response);
    Ok(())
}
