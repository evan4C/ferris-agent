# ferris-agent

<p align="center">
 | <a href="README.md">English</a> | <a href="README_zh-CN.md">简体中文</a> | 
</p>

`ferris-agent` is a lightweight AI agent project written in Rust. It uses the DeepSeek Chat Completions API and supports multi-turn conversations and tool calls. It currently includes file read/write, shell command, and Git status tools, with a tool registry that controls which tools are available to the agent.

## Features

- Chat with the DeepSeek API using the `deepseek-flash` and `deepseek-v4-pro` models.
- `Conversation` stores the system prompt, user messages, assistant replies, and tool results for subsequent turns.
- `Agent` sends model requests, handles tool calls, and returns tool results to the model. The number of tool-call rounds is limited by `deepseek.max_steps`.
- `ToolRegistry` registers tools, generates the tool definitions required by the API, and dispatches model-issued calls.
- Built-in tools: `read_file`, `write_file`, `bash`, and `git_status`.

## Architecture

```text
User input
   │
   ▼
Conversation ── stores the full message history
   │
   ▼
Agent ── sends model requests and drives the tool-call loop
   ├── DeepSeekClient ── sends Chat Completions HTTP requests
   └── ToolRegistry ── executes tools and returns results
		  ├── Files: read_file / write_file
		  ├── Shell: bash
		  └── Git: git_status
```

- **`Conversation`** manages a conversation's message history and provides `send`, `history`, and `reset`.
- **`Agent`** coordinates model requests and the tool-call loop; it does not own persistent conversation history.
- **`DeepSeekClient`** sends requests to the configured DeepSeek endpoint and parses responses.
- **`ToolRegistry`** manages available tools and their dispatch; `ToolContext` provides tools with a working directory and environment variables.

## Configuration

Install the Rust toolchain and provide a DeepSeek API key. Set it in a `.env` file in the project root:

```dotenv
DEEPSEEK__API_KEY=your-api-key
```

The project reads `config.toml` and environment variables. The runtime currently uses the following DeepSeek settings:

```toml
[deepseek]
base_url = "https://api.deepseek.com/chat/completions"
max_steps = 10
```

You can also set the corresponding environment variables `DEEPSEEK__BASE_URL` and `DEEPSEEK__MAX_STEPS`. Do not commit your API key; `.env` is ignored by the repository's `.gitignore`.

## Run

```sh
cargo run
```

The example program sends one request to the model and prints the final reply. It uses the current directory as the tool working directory and registers only the file read/write tools by default.

## Use as a Rust Library

```rust,no_run
use ferris_agent::{Conversation, DeepSeekClient};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let client = Arc::new(DeepSeekClient::new());
	let mut conversation = Conversation::new(client);

	let reply = conversation.send("Read README.md and summarize it.").await?;
	println!("{reply}");

	// The next turn includes the earlier messages and tool results.
	let follow_up = conversation.send("Make the summary shorter.").await?;
	println!("{follow_up}");

	Ok(())
}
```

To enable additional tools, build a registry and pass it to the conversation:

```rust,no_run
use ferris_agent::{Conversation, DeepSeekClient};
use ferris_agent::tool::registry::ToolRegistry;
use std::sync::Arc;

let registry = ToolRegistry::builder()
	.filesystem()
	.shell()
	.git()
	.build();

let conversation = Conversation::new(Arc::new(DeepSeekClient::new()))
	.with_tool_registry(registry);
```

`history()` provides read-only access to the message history. `reset()` clears the conversation turns while keeping the initial system prompt. When changing the tool registry, explicitly include every tool you want to keep enabled.

## Tools and Security

`Agent::new` enables `read_file` and `write_file` by default. Shell and Git tools are only exposed to the model when `.shell()` and `.git()` are respectively called on the registry. File tools join the supplied path to the working directory; the Shell tool executes model-generated commands through `sh -c` and inherits the process environment variables.

These tools are not currently sandboxed: file paths are not restricted to the working directory, and shell commands do not require approval. Enable them only in environments you trust, and be aware that tools may read, modify, or delete local data. The `[tools]` and `[model]` settings in `config.toml` are not yet connected to the runtime. The actual working directory is the process's current directory at startup, and the default model is selected by `ChatOptions`.

## Development

```sh
cargo test
```

The main code is under `src/`:

```text
src/
├── agent/        # Agent orchestration and tool-call loop
├── api/          # DeepSeek client, requests, responses, and message types
├── config/       # Configuration loading and system prompt
├── conversation/ # Multi-turn conversations and options
└── tool/         # Tool interface, registry, and built-in tools
```
