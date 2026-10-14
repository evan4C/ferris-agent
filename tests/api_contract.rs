#[path = "support/mod.rs"]
mod support;

use ferris_agent::agent::{AgentRequest, ChatBackend};
use ferris_agent::config::DeepSeekConfig;
use ferris_agent::{ChatCompletionRequest, DeepSeekClient, Message, Model};
use serde_json::{Value, json};
use support::spawn_http_response;

fn response_body(message: Value) -> Vec<Vec<u8>> {
    vec![
        json!({
            "id": "response-1",
            "object": "chat.completion",
            "created": 1,
            "model": "deepseek-flash",
            "choices": [{
                "index": 0,
                "finish_reason": "stop",
                "message": message
            }],
            "usage": {
                "prompt_tokens": 5,
                "completion_tokens": 3,
                "total_tokens": 8
            }
        })
        .to_string()
        .into_bytes(),
    ]
}

fn client(base_url: String) -> DeepSeekClient {
    DeepSeekClient::new(
        DeepSeekConfig {
            base_url,
            ..DeepSeekConfig::default()
        },
        "test-token".into(),
    )
    .unwrap()
}

#[tokio::test]
async fn sends_chat_request_and_parses_json_response() {
    let (url, server) = spawn_http_response(
        "200 OK",
        "application/json",
        response_body(json!({"role": "assistant", "content": "hello"})),
    )
    .await;
    let client = client(url);
    let turn = client
        .complete(AgentRequest {
            model: Model::Flash,
            stream: false,
            thinking: false,
            max_tokens: Some(17),
            messages: vec![Message::user("hi")],
            tools: Vec::new(),
        })
        .await
        .unwrap();
    let captured = server.await.unwrap();
    let body: Value = serde_json::from_slice(&captured.body).unwrap();

    assert!(
        captured
            .headers
            .to_ascii_lowercase()
            .contains("authorization: bearer test-token")
    );
    assert_eq!(body["model"], "deepseek-flash");
    assert_eq!(body["max_tokens"], 17);
    assert_eq!(
        body["messages"].as_array().unwrap().last().unwrap()["content"],
        "hi"
    );
    assert_eq!(turn.message.content.as_deref(), Some("hello"));
    assert_eq!(turn.usage.unwrap().total_tokens, 8);
}

#[tokio::test]
async fn reports_non_success_http_responses() {
    let (url, server) = spawn_http_response(
        "429 Too Many Requests",
        "application/json",
        vec![br#"{"error":"rate limited"}"#.to_vec()],
    )
    .await;
    let client = client(url);
    let request = ChatCompletionRequest::new(&client).user("hi");

    let error = client.http_request(&request).await.unwrap_err();
    server.await.unwrap();
    assert!(error.to_string().contains("429 Too Many Requests"));
    assert!(error.to_string().contains("rate limited"));
}

#[tokio::test]
async fn rejects_success_responses_without_choices() {
    let (url, server) = spawn_http_response(
        "200 OK",
        "application/json",
        vec![
            br#"{"id":"response-1","object":"chat.completion","created":1,"model":"deepseek-flash","choices":[]}"#
                .to_vec(),
        ],
    )
    .await;
    let client = client(url);
    let request = ChatCompletionRequest::new(&client).user("hi");

    let error = client.http_request(&request).await.unwrap_err();
    server.await.unwrap();
    assert!(error.to_string().contains("no choices"));
}

#[tokio::test]
async fn assembles_streamed_content_and_tool_call_fragments() {
    let events = [
        json!({"choices":[{"delta":{"role":"assistant","content":"hel","tool_calls":[{"index":0,"id":"call-1","function":{"name":"read_","arguments":"{\"path\":"}}]}}]}),
        json!({"choices":[{"delta":{"content":"lo","tool_calls":[{"index":0,"function":{"name":"file","arguments":"\"note.txt\"}"}}]}}]}),
        json!({"choices":[],"usage":{"prompt_tokens":5,"completion_tokens":3,"total_tokens":8}}),
    ];
    let mut stream_chunks: Vec<Vec<u8>> = events
        .iter()
        .map(|event| format!("data: {event}\n").into_bytes())
        .collect();
    stream_chunks.push(b"data: [DONE]\n".to_vec());
    let (url, server) = spawn_http_response("200 OK", "text/event-stream", stream_chunks).await;
    let client = client(url);
    let request = ChatCompletionRequest::new(&client)
        .stream(true)
        .messages(vec![Message::user("read my note")]);
    let turn = client.http_request(&request).await.unwrap();
    server.await.unwrap();

    assert_eq!(turn.message.content.as_deref(), Some("hello"));
    assert_eq!(turn.message.tool_calls.len(), 1);
    assert_eq!(turn.message.tool_calls[0].function.name, "read_file");
    assert_eq!(
        turn.message.tool_calls[0].function.arguments,
        r#"{"path":"note.txt"}"#
    );
    assert_eq!(turn.usage.unwrap().total_tokens, 8);
}
