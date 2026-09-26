use ferris_agent::DeepSeekClient;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut current_step: u8 = 0;
    let agent = DeepSeekClient::new();
    let mut request_body = agent
        .chat()
        .user("what is the capital of China?");

    // the main agent loop
    loop {
        current_step += 1;
        if current_step > 10 {
            eprintln!("Maximum steps reached");
            break;
        }

        // Step 1: get llm response and push it to the conversation history
        let llm_output: String = agent.query_llm(&request_body).await?;
        println!("LLM output: {}", llm_output);

        request_body = request_body.assistant(llm_output.clone());

        // Step 2: parse tool calls and exit the loop if there are no tool calls
        let tool_calls = agent.parse_tool_calls(&llm_output);
        println!("Parsed tool calls: {:?}", tool_calls);
        
        if tool_calls.is_empty() {
            break;
        }

        // Step 3: execute tool calls and push the results to the conversation history
        let action = tool_calls[0].clone();
        let action_result = agent.execute_action(action);
        println!("Action result: {}", action_result);

        request_body = request_body.tool(action_result.clone());
    }
    Ok(())
}
