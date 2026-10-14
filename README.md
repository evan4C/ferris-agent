# ferris-agent

<p align="center">
 | <a href="README.md">English</a> | <a href="README_zh-CN.md">简体中文</a> | 
</p>

`ferris-agent` is a lightweight AI agent project written in Rust. It uses the DeepSeek Chat Completions API and supports multi-turn conversations and tool calls. It currently includes file read/write, shell command, and Git status tools, with a tool registry that controls which tools are available to the agent.

## Features

- Chat with the DeepSeek API using the `deepseek-flash` and `deepseek-v4-pro` models.
- `Agent` stores message history, sends model requests, handles tool calls, and returns tool results to the model. The number of tool-call rounds is limited by `agent.max_iterations`.
- `ChatBackend` abstracts a model completion; the production implementation uses `DeepSeekClient`, while tests can provide a scripted backend.
- `config` manages the application configuration, including DeepSeek API settings and agent parameters. The app configuration file is located in the platform-specific user configuration directory.
- `ToolRegistry` registers tools, generates the tool definitions required by the API, and dispatches model-issued calls.
- Built-in tools: `read_file`, `write_file`, `bash`, and `git_status`.
- Assistant Markdown replies are rendered in the terminal with `termimad`.

## Architecture

```text
User input
   │
   ▼
Agent ── stores message history and drives the tool-call loop
   ├── ChatBackend ── model-completion interface
   │   └── DeepSeekClient ── sends Chat Completions HTTP requests
   └── ToolRegistry ── executes tools and returns results
       ├── Files: read_file / write_file
       ├── Shell: bash
       └── Git: git_status
```

- **`Agent`** manages message history with `add_user_message`, `history`, and `reset_history`, and coordinates model requests and tool execution.
- **`ChatBackend`** provides the completion interface used by the agent. `DeepSeekClient` implements it for real API requests; tests can inject a fake backend with `Agent::with_backend`.
- **`DeepSeekClient`** sends requests to the configured DeepSeek endpoint and parses responses.
- **`ToolRegistry`** manages available tools and their dispatch; `ToolContext` provides tools with a working directory and environment variables.

## Configuration

The application stores non-sensitive settings in the platform-specific user configuration directory and the API key in the `FERRIS_AGENT_DEEPSEEK_API_KEY` environment variable, which works on every platform. Initialize the configuration with:

```sh
cargo run -- init
```

The generated `config.toml` contains only non-sensitive settings. Its defaults are also available in [`config.example.toml`](config.example.toml):

The file is stored under the platform-specific user configuration directory, such as `~/.config/ferris-agent/config.toml` on Linux or `~/Library/Application Support/ferris-agent/config.toml` on macOS.

Edit `config.toml` directly to change settings.

```toml
[deepseek]
model = "deepseek-chat"
base_url = "https://api.deepseek.com/chat/completions"

[agent]
max_iterations = 10
```

Set the API key before running:

```sh
export FERRIS_AGENT_DEEPSEEK_API_KEY="your-api-key"
cargo run
```

## Run

```sh
cargo run
```

The example program sends one request to the model and prints the final reply. It uses the current directory as the tool working directory and registers only the file read/write tools by default.

## Use as a Rust Library

```rust,no_run
use anyhow::Result;
use clap::Parser;
use ferris_agent::{Cli, app};

#[tokio::main]
async fn main() -> Result<()> {
    app::exec(Cli::parse()).await
}
```

To use the agent directly, create a client and pass a tool registry:

```rust,no_run
use anyhow::Result;
use ferris_agent::config::{AppConfig, credential};
use ferris_agent::{Agent, DeepSeekClient};
use ferris_agent::tool::registry::ToolRegistry;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;
    let api_key = credential::get_api_key("deepseek")?;
    let client = Arc::new(DeepSeekClient::new(config.deepseek, api_key)?);
    let registry = ToolRegistry::builder()
        .filesystem()
        .shell()
        .git()
        .build();

    let mut agent = Agent::new(client);
    agent
        .with_max_iterations(config.agent.max_iterations)
        .with_tool_registry(registry)
        .add_user_message("Summarize the current Git status.".into());
    let reply = agent.run().await?;
    println!("{}", reply.content);
    Ok(())
}
```

`history()` provides read-only access to the agent's message history. `reset_history()` clears the turns while keeping the initial system prompt. When changing the tool registry, explicitly include every tool you want to keep enabled.

## Tools and Security

`Agent::new` enables `read_file` and `write_file` by default. Shell and Git tools are only exposed to the model when `.shell()` and `.git()` are respectively called on the registry. File tools join the supplied path to the working directory; the Shell tool executes model-generated commands through `sh -c` and inherits the process environment variables.

These tools are not currently sandboxed: file paths are not restricted to the working directory, and shell commands do not require approval. Enable them only in environments you trust, and be aware that tools may read, modify, or delete local data. The configured model, tool-call limit, and optional `tools.workspace` are used by the runtime; an explicit `--model` argument overrides the configured model.

## Development

### Release workflow

To create a new release, tag the commit with a version number following the pattern `v*.*.*` and push the tag. The GitHub Actions workflow will automatically build the project for multiple targets, package the binaries, and publish a release.

### Running Tests

```sh
cargo test --all
cargo fmt --all -- --check
cargo clippy --all-targets
```

The test suite does not require API credentials or external services. Integration tests use a local HTTP server for API contracts and a scripted backend for agent workflows. See the [testing guide](docs/testing.md) for test layers and guidance for adding coverage.

### Project Structure

The main code is under `src/`:

```text
src/
├── agent/        # Agent orchestration and tool-call loop
├── api/          # DeepSeek client, requests, responses, and message types
├── config/       # Configuration loading and system prompt
└── tool/         # Tool interface, registry, and built-in tools
tests/
├── agent_workflows.rs # Deterministic agent workflow tests
├── api_contract.rs    # Local HTTP API contract tests
└── support/           # Shared test-only helpers
docs/
└── testing.md         # Testing guide and conventions
```
