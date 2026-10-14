use async_trait::async_trait;
use ferris_agent::Agent;
use ferris_agent::agent::{AgentRequest, ChatBackend};
use ferris_agent::api::{ApiTurn, ChatMessage, ChatUsage, DeepSeekError};
use ferris_agent::tool::ToolCall;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::sync::Mutex;

struct ScriptedBackend {
    turns: Mutex<VecDeque<ApiTurn>>,
    requests: Mutex<Vec<Vec<Value>>>,
}

impl ScriptedBackend {
    fn new(turns: Vec<ApiTurn>) -> Self {
        Self {
            turns: Mutex::new(turns.into()),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn request_history(&self) -> Vec<Vec<Value>> {
        self.requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl ChatBackend for ScriptedBackend {
    async fn complete(&self, request: AgentRequest) -> Result<ApiTurn, DeepSeekError> {
        self.requests.lock().unwrap().push(
            request
                .messages
                .iter()
                .map(|message| serde_json::to_value(message).unwrap())
                .collect(),
        );
        self.turns
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| DeepSeekError::Api("scripted backend has no response".into()))
    }
}

fn turn(content: &str, usage: Option<ChatUsage>) -> ApiTurn {
    ApiTurn {
        message: ChatMessage {
            role: "assistant".into(),
            content: Some(content.into()),
            reasoning_content: None,
            tool_calls: Vec::new(),
        },
        usage,
    }
}

fn tool_turn(call: ToolCall, usage: Option<ChatUsage>) -> ApiTurn {
    ApiTurn {
        message: ChatMessage {
            role: "assistant".into(),
            content: None,
            reasoning_content: None,
            tool_calls: vec![call],
        },
        usage,
    }
}

fn usage(prompt_tokens: u32, completion_tokens: u32) -> ChatUsage {
    ChatUsage {
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
    }
}

#[tokio::test]
async fn returns_final_reply_and_records_history() {
    let backend = std::sync::Arc::new(ScriptedBackend::new(vec![turn("done", Some(usage(4, 2)))]));
    let mut agent = Agent::with_backend(backend.clone());
    agent.add_user_message("hello".into());

    let reply = agent.run().await.unwrap();
    let history = agent.history();

    assert_eq!(reply.content, "done");
    assert_eq!(reply.usage.unwrap().total_tokens, 6);
    assert_eq!(history.len(), 3);
    assert_eq!(backend.request_history().len(), 1);
}

#[tokio::test]
async fn executes_tool_and_accumulates_usage_across_rounds() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("note.txt"), "tool output").unwrap();
    let backend = std::sync::Arc::new(ScriptedBackend::new(vec![
        tool_turn(
            ToolCall::new(
                "call-1".into(),
                "read_file".into(),
                json!({"path":"note.txt"}).to_string(),
            ),
            Some(usage(3, 1)),
        ),
        turn("finished", Some(usage(5, 2))),
    ]));
    let mut agent = Agent::with_backend(backend.clone());
    agent
        .with_workspace(directory.path().to_path_buf())
        .add_user_message("read note.txt".into());

    let reply = agent.run().await.unwrap();
    let requests = backend.request_history();
    let second_history = requests.get(1).unwrap();
    let tool_result = second_history
        .iter()
        .find(|message| message["role"] == "tool")
        .unwrap();

    assert_eq!(reply.content, "finished");
    assert_eq!(reply.usage.unwrap().total_tokens, 11);
    assert_eq!(requests.len(), 2);
    assert_eq!(tool_result["tool_call_id"], "call-1");
    assert_eq!(tool_result["content"], "tool output");
}

#[tokio::test]
async fn reports_tool_errors_in_the_follow_up_messages() {
    let directory = tempfile::tempdir().unwrap();
    let backend = std::sync::Arc::new(ScriptedBackend::new(vec![
        tool_turn(
            ToolCall::new(
                "call-missing".into(),
                "read_file".into(),
                json!({"path":"missing.txt"}).to_string(),
            ),
            None,
        ),
        turn("I could not read the file.", None),
    ]));
    let mut agent = Agent::with_backend(backend.clone());
    agent
        .with_workspace(directory.path().to_path_buf())
        .add_user_message("read missing.txt".into());

    agent.run().await.unwrap();
    let requests = backend.request_history();
    let tool_result = requests[1]
        .iter()
        .find(|message| message["role"] == "tool")
        .unwrap();

    assert_eq!(tool_result["tool_call_id"], "call-missing");
    assert!(
        tool_result["content"]
            .as_str()
            .unwrap()
            .starts_with("Tool failed:")
    );
}

#[tokio::test]
async fn enforces_maximum_tool_call_rounds() {
    let backend = std::sync::Arc::new(ScriptedBackend::new(vec![
        tool_turn(
            ToolCall::new("call-1".into(), "unknown".into(), "{}".into()),
            None,
        ),
        tool_turn(
            ToolCall::new("call-2".into(), "unknown".into(), "{}".into()),
            None,
        ),
    ]));
    let mut agent = Agent::with_backend(backend.clone());
    agent
        .with_max_iterations(1)
        .add_user_message("keep working".into());

    let error = match agent.run().await {
        Ok(_) => panic!("agent should stop after reaching the tool-call limit"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("Maximum tool-call rounds reached")
    );
    assert_eq!(backend.request_history().len(), 2);
}
