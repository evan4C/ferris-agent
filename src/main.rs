use ferris_agent::deepseek;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut agent = deepseek::DSAgent::new();
    let system_prompt = String::from("You are a helpful assistant.");
    agent.request_body.add_system_prompt(system_prompt);

    // example of user input
    let user_input = String::from("Hello, can you help me with a task?");
    agent.request_body.add_user_message(user_input);

    // the main agent loop
    loop {

        // Step 1: get llm response and push it to the conversation history
        let llm_output: String = agent.query_llm().await?;
        println!("LLM output: {}", llm_output);

        agent.request_body.add_assistant_message(llm_output.clone());

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

        agent.request_body.add_toolcall_message(action_result.clone());
    }
    Ok(())
}
