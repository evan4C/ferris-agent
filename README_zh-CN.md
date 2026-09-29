# ferris-agent

`ferris-agent` 是一个使用 Rust 编写的轻量级 AI Agent 项目，基于 DeepSeek Chat Completions API，支持多轮对话和工具调用。它当前包含文件读写、Shell 命令和 Git 状态工具，并通过工具注册表控制哪些工具对 Agent 可用。

## 功能

- 通过 DeepSeek API 进行对话，支持 `deepseek-flash` 和 `deepseek-v4-pro` 模型。
- `Conversation` 保存系统提示词、用户消息、助手回复和工具调用结果，供后续轮次继续使用。
- `Agent` 负责请求模型、处理工具调用并将工具结果交回模型；工具调用轮数由 `deepseek.max_steps` 限制。
- `ToolRegistry` 注册工具、生成 API 所需的工具定义，并分发模型发出的调用。
- 内置工具：`read_file`、`write_file`、`bash` 和 `git_status`。

## 架构

```text
用户输入
   │
   ▼
Conversation ── 保存完整消息历史
   │
   ▼
Agent ── 请求模型、驱动工具调用循环
   ├── DeepSeekClient ── 发送 Chat Completions HTTP 请求
   └── ToolRegistry ── 执行工具并返回结果
          ├── 文件：read_file / write_file
          ├── Shell：bash
          └── Git：git_status
```

- **`Conversation`** 管理一次会话的消息历史，并提供 `send`、`history` 和 `reset`。
- **`Agent`** 协调模型请求和工具调用循环，不负责长期保存会话历史。
- **`DeepSeekClient`** 负责向配置的 DeepSeek endpoint 发送请求并解析响应。
- **`ToolRegistry`** 管理可用工具及其调用；`ToolContext` 向工具提供工作目录和环境变量。

## 环境配置

需要安装 Rust 工具链，并准备一个 DeepSeek API key。在项目根目录的 `.env` 中设置：

```dotenv
DEEPSEEK__API_KEY=your-api-key
```

项目会读取 `config.toml` 和环境变量。当前运行时代码使用以下 DeepSeek 配置：

```toml
[deepseek]
base_url = "https://api.deepseek.com/chat/completions"
max_steps = 10
```

也可以通过 `DEEPSEEK__BASE_URL` 和 `DEEPSEEK__MAX_STEPS` 设置对应环境变量。请勿将 API key 提交到版本控制；仓库的 `.gitignore` 已忽略 `.env`。

## 运行

```sh
cargo run
```

当前示例程序会向模型发送一条请求，并打印最终回复。它使用当前目录作为工具工作目录，默认只注册文件读写工具。

## 作为 Rust 库使用

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

需要其他工具时，构造注册表并传入会话：

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

`history()` 提供只读消息历史；`reset()` 清除对话轮次并保留初始系统提示词。更改工具注册表时请重新显式加入希望保留的工具。

## 工具与安全

`Agent::new` 默认启用 `read_file` 和 `write_file`。Shell 与 Git 工具只有在注册表中分别调用 `.shell()` 和 `.git()` 后才会提供给模型。文件工具以工作目录拼接传入路径；Shell 工具通过 `sh -c` 执行模型生成的命令，并继承进程环境变量。

这些工具目前不是安全沙箱：文件路径没有限制在工作目录内，Shell 命令也没有权限审批机制。只应在你信任的环境中启用，并注意工具可能读取、修改或删除本机数据。`config.toml` 中的 `[tools]`、`[model]` 配置目前尚未接入运行时；实际工作目录是启动程序时的当前目录，默认模型由 `ChatOptions` 决定。

## 开发

```sh
cargo test
```

主要代码位于 `src/`：

```text
src/
├── agent/        # Agent 调度与工具调用循环
├── api/          # DeepSeek 客户端、请求、响应和消息类型
├── config/       # 配置加载与系统提示词
├── conversation/ # 多轮会话及选项
└── tool/          # 工具接口、注册表和内置工具
```
