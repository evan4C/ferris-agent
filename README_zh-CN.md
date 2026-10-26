# ferris-agent

`ferris-agent` 是一个使用 Rust 编写的轻量级 AI Agent 项目，基于 DeepSeek Chat Completions API，支持多轮对话和工具调用。它当前包含文件读写、Shell 命令和 Git 状态工具，并通过工具注册表控制哪些工具对 Agent 可用。

## 功能

- 通过 DeepSeek API 进行对话，支持 `deepseek-flash` 和 `deepseek-v4-pro` 模型。
- `Agent` 保存消息历史、请求模型、处理工具调用并将工具结果交回模型；工具调用轮数由 `agent.max_iterations` 限制。
- `ChatBackend` 抽象单次模型请求：生产环境使用 `DeepSeekClient`，测试可注入脚本化后端。
- `config` 管理应用配置，包括 DeepSeek API 设置和 Agent 参数。app 配置文件位于平台对应的用户配置目录中。
- `ToolRegistry` 注册工具、生成 API 所需的工具定义，并分发模型发出的调用。
- 内置工具：`read_file`、`write_file`、`bash` 和 `git_status`。
- 使用 `termimad` 在终端中渲染助手的 Markdown 回复。

## 架构

```text
用户输入
   │
   ▼
Agent ── 保存消息历史并驱动工具调用循环
   ├── ChatBackend ── 模型请求接口
   │   └── DeepSeekClient ── 发送 Chat Completions HTTP 请求
   └── ToolRegistry ── 执行工具并返回结果
       ├── 文件：read_file / write_file
       ├── Shell：bash
       └── Git：git_status
```

- **`Agent`** 通过 `add_user_message`、`history` 和 `reset_history` 管理消息历史，并协调模型请求与工具执行。
- **`ChatBackend`** 提供 Agent 使用的模型请求接口。`DeepSeekClient` 实现真实 API 请求；测试可通过 `Agent::with_backend` 注入模拟后端。
- **`DeepSeekClient`** 负责向配置的 DeepSeek endpoint 发送请求并解析响应。
- **`ToolRegistry`** 管理可用工具及其调用；`ToolContext` 向工具提供工作目录和环境变量。

## 环境配置

非敏感设置保存在平台对应的用户配置目录中，API key 通过环境变量 `FERRIS_AGENT_DEEPSEEK_API_KEY` 提供，适用于所有平台。初始化配置：

```sh
cargo run -- init
```

生成的 `config.toml` 只包含非敏感设置。默认值也见 [`config.example.toml`](config.example.toml)：

配置文件位于平台对应的用户配置目录，例如 Linux 上的 `~/.config/ferris-agent/config.toml`，或 macOS 上的 `~/Library/Application Support/ferris-agent/config.toml`。

如需更改设置，请直接编辑 `config.toml`。

```toml
[deepseek]
model = "deepseek-chat"
base_url = "https://api.deepseek.com/chat/completions"

[agent]
max_iterations = 10
```

运行前通过环境变量 `FERRIS_AGENT_DEEPSEEK_API_KEY` 设置密钥：

```sh
export FERRIS_AGENT_DEEPSEEK_API_KEY="your-api-key"
cargo run
```

## 运行

```sh
cargo run
```

当前示例程序会向模型发送一条请求，并打印最终回复。它使用当前目录作为工具工作目录，默认只注册文件读写工具。

## 作为 Rust 库使用

```rust,no_run
use anyhow::Result;
use clap::Parser;
use ferris_agent::{Cli, app};

#[tokio::main]
async fn main() -> Result<()> {
    app::exec(Cli::parse()).await
}
```

如需直接使用 Agent，创建客户端并传入工具注册表：

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

`history()` 提供 Agent 的只读消息历史；`reset_history()` 清除对话轮次并保留初始系统提示词。更改工具注册表时请重新显式加入希望保留的工具。

## 工具与安全

`Agent::new` 默认启用 `read_file` 和 `write_file`。Shell 与 Git 工具只有在注册表中分别调用 `.shell()` 和 `.git()` 后才会提供给模型。文件工具以工作目录拼接传入路径；Shell 工具通过 `sh -c` 执行模型生成的命令，并继承进程环境变量。

这些工具目前不是安全沙箱：文件路径没有限制在工作目录内，Shell 命令也没有权限审批机制。只应在你信任的环境中启用，并注意工具可能读取、修改或删除本机数据。运行时会使用配置的模型、工具调用轮数和可选的 `tools.workspace`；显式传入 `--model` 时会覆盖配置模型。

## 开发

```sh
cargo test --all
cargo fmt --all -- --check
cargo clippy --all-targets
```

测试无需 API 密钥或外部服务：集成测试使用本地 HTTP 服务验证 API 契约，并使用脚本化后端测试 Agent 工作流。测试分层及新增测试的说明请参阅[测试指南](docs/testing.md)。

主要代码位于 `src/`：

```text
src/
├── agent/        # Agent 调度与工具调用循环
├── api/          # DeepSeek 客户端、请求、响应和消息类型
├── config/       # 配置加载与系统提示词
└── tool/          # 工具接口、注册表和内置工具
tests/
├── agent_workflows.rs # Agent 工作流测试
├── api_contract.rs    # 本地 HTTP API 契约测试
└── support/           # 测试专用共享辅助代码
docs/
└── testing.md         # 测试指南与约定
```
