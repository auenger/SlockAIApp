# Feature: feat-agent-runtime-exec 多 Runtime 对话执行

## Basic Information
- **ID**: feat-agent-runtime-exec
- **Name**: 多 Runtime 对话执行
- **Priority**: 75
- **Size**: S
- **Dependencies**: feat-agent-runtime-model, feat-agent-runtime-ui
- **Parent**: feat-agent-runtime-select
- **Children**: (none)
- **Created**: 2026-04-10

## Description

实现对话执行时根据 Agent 绑定的 runtime_type 自动路由到对应的 runtime 实现。每个 Thread/Channel 对话在发送消息时，读取 Agent 的 runtime_type，调用对应 Runtime 的 execute 方法，实现真正的多 runtime 对话。

参考 `reference/AINative/` 的 Session-based 执行模式和 Smart Routing 引擎。

## User Value Points

1. **自动 Runtime 路由**: 用户只需选择 Agent 对话，系统自动使用正确的 runtime 后端
2. **Session 管理 Per-Runtime**: 每个 runtime 独立管理 session，支持上下文连续对话

## Context Analysis

### Reference Code
- `src-tauri/src/commands/chat.rs` — 现有对话 commands
- `src-tauri/src/runtime/claude.rs` — Claude Code 执行逻辑
- `src/lib/useThreadChat.ts` — 前端 Thread 对话 hook
- `src/lib/useChannel.ts` — 前端 Channel 多 agent 对话 hook
- `reference/AINative/neuro-syntax-ide/src/lib/useSmartRouter.ts` — Smart Routing 参考

### Related Features
- `feat-agent-runtime-model` (依赖) — Runtime trait 和 registry
- `feat-agent-runtime-ui` (依赖) — Agent 绑定 runtime_type

## Technical Solution

### 1. Runtime 路由层

在对话执行时增加 runtime 路由逻辑：

```rust
// commands/chat.rs
#[tauri::command]
pub async fn send_thread_message(
    app: AppHandle,
    thread_id: String,
    message: String,
) -> Result<(), String> {
    // 1. 获取 thread 关联的 agent
    let agent = workspace_manager.get_thread_agent(&thread_id)?;

    // 2. 根据 agent 的 runtime_type 获取 runtime
    let runtime = RuntimeRegistry::get_runtime(&agent.runtime_type)
        .ok_or("Runtime not available")?;

    // 3. 检查 runtime 健康
    let health = runtime.health_check().await?;
    if !health.is_available {
        return Err(format!("Runtime {} is not available", runtime.name()));
    }

    // 4. 获取或创建 session
    let session_id = get_or_create_session(&thread_id, &runtime).await?;

    // 5. 执行消息
    let params = ExecuteParams {
        session_id: Some(session_id),
        message,
        context: Some(load_context(&thread_id)?),
    };

    // 6. 流式输出
    match runtime.execute(params).await {
        Ok(stream) => {
            // 通过 app.emit 发送流式 chunk
            handle_stream(app, thread_id, stream).await
        }
        Err(e) => Err(format!("Execution failed: {}", e)),
    }
}
```

### 2. Session 管理 Per-Runtime

```rust
// runtime/session_manager.rs
pub struct SessionManager {
    sessions: HashMap<String, RuntimeSession>,  // thread_id -> session
}

struct RuntimeSession {
    runtime_type: RuntimeType,
    session_id: String,
    created_at: DateTime<Utc>,
    last_active: DateTime<Utc>,
}

impl SessionManager {
    pub async fn get_or_create(
        &mut self,
        thread_id: &str,
        runtime_type: &RuntimeType,
    ) -> Result<String> {
        if let Some(session) = self.sessions.get(thread_id) {
            if &session.runtime_type == runtime_type {
                return Ok(session.session_id.clone());
            }
            // Runtime 类型变了，需要重建 session
        }
        // 创建新 session
        let runtime = RuntimeRegistry::get_runtime(runtime_type)?;
        let session = runtime.create_session(SessionConfig::default()).await?;
        self.sessions.insert(thread_id.to_string(), RuntimeSession {
            runtime_type: runtime_type.clone(),
            session_id: session.id,
            created_at: Utc::now(),
            last_active: Utc::now(),
        });
        Ok(session.id)
    }
}
```

### 3. 前端 Thread Chat 适配

```typescript
// useThreadChat.ts — 修改
async function sendMessage(threadId: string, content: string) {
  // 发送消息时不再关心 runtime 类型
  // 后端根据 agent.runtime_type 自动路由
  await invoke('send_thread_message', { threadId, message: content });

  // 监听流式响应（不变）
  const unlisten = listen('agent://chunk', (event) => {
    // 处理流式 chunk
  });
}
```

### 4. 错误处理与降级

```rust
// 当首选 runtime 不可用时的降级策略
pub async fn execute_with_fallback(
    agent: &AgentConfig,
    message: &str,
) -> Result<ExecuteResult> {
    // 尝试首选 runtime
    match try_execute(&agent.runtime_type, message).await {
        Ok(result) => Ok(result),
        Err(_) => {
            // 首选不可用，尝试降级
            for fallback in get_fallback_chain(&agent.runtime_type) {
                if let Ok(result) = try_execute(&fallback, message).await {
                    return Ok(result);
                }
            }
            Err("All runtimes unavailable".into())
        }
    }
}
```

### 5. Channel 多 Agent 对话路由

```rust
// channel 执行时，@mention 的 agent 各自路由到对应 runtime
#[tauri::command]
pub async fn send_channel_message(
    app: AppHandle,
    channel_id: String,
    message: String,
    mentioned_agents: Vec<String>,
) -> Result<(), String> {
    for agent_id in mentioned_agents {
        let agent = workspace_manager.get_agent(&agent_id)?;
        let runtime = RuntimeRegistry::get_runtime(&agent.runtime_type)?;
        // 并发执行各 agent
        tokio::spawn(execute_for_agent(app.clone(), agent, message.clone()));
    }
    Ok(())
}
```

## Acceptance Criteria (Gherkin)

### User Story
As a user, I want my conversations to automatically use the correct AI runtime based on the agent I selected, so I don't need to worry about the underlying implementation.

### Scenarios

```gherkin
Scenario: Thread message routes to agent's runtime
  Given an agent "CodeReviewer" with runtime_type "claude_code"
  And a thread associated with "CodeReviewer"
  When the user sends a message in the thread
  Then the message is executed via ClaudeCodeRuntime

Scenario: Channel with multiple agents uses different runtimes
  Given a channel with @ClaudeAgent (claude_code) and @CodexAgent (codex)
  When the user sends "review this code @ClaudeAgent @CodexAgent"
  Then ClaudeAgent's response comes from ClaudeCodeRuntime
  And CodexAgent's response comes from CodexRuntime

Scenario: Session persists within same thread
  Given a thread with agent using claude_code runtime
  When the user sends message A and gets session_id "sess_123"
  And the user sends message B in the same thread
  Then message B uses session resume with "sess_123"

Scenario: Runtime unavailable shows clear error
  Given an agent with runtime_type "codex"
  And Codex CLI is not installed
  When the user sends a message to this agent
  Then an error message shows "Codex runtime is not available"
  And suggests installation command
```

### General Checklist
- [x] send_thread_message 根据 agent.runtime_type 路由
- [x] Session 管理 per-thread，支持 resume
- [x] Channel 多 agent 并发执行各自路由
- [x] Runtime 不可用时显示明确错误信息
- [x] 流式输出通过 Tauri event 正常工作

## Merge Record

- **Completed**: 2026-04-10T20:00:00+08:00
- **Merged Branch**: feature/feat-agent-runtime-exec
- **Merge Commit**: a061f6e
- **Archive Tag**: feat-agent-runtime-exec-20260410
- **Conflicts**: none
- **Verification**: passed (63/63 tests, 4/4 Gherkin scenarios)
- **Stats**: 1 commit, 4 files changed, ~15min duration
