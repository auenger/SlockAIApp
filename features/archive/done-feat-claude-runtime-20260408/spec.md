# Feature: feat-claude-runtime Claude Code Runtime Agent

## Basic Information
- **ID**: feat-claude-runtime
- **Name**: Claude Code Runtime Agent
- **Priority**: 70
- **Size**: M
- **Dependencies**: feat-project-init
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-08

## Description

实现 SlockAI 的 Agent Runtime 层，以 Claude Code CLI 作为首个 runtime agent。参考 AINative 项目的 `lib.rs` 实现，包含统一的 `AgentRuntime` trait 抽象、Claude Code CLI 子进程调用、`stream-json` 格式的流式响应解析、会话管理（`--resume`）、以及基于 Tauri Event 的实时前端推送。

核心目标：在 Rust 后端实现一个可扩展的 Agent Runtime 框架，支持 Claude Code CLI 作为第一个 runtime，通过 Tauri IPC 将流式响应实时推送到前端渲染。

## User Value Points

1. **Agent Runtime 统一抽象** — 通过 `AgentRuntime` trait 定义统一接口，未来可无缝扩展 Codex / Gemini HTTP 等多种 runtime，前端无需感知底层差异
2. **Claude Code CLI 集成** — 将 Claude Code CLI 作为子进程调用，支持流式输出、会话恢复、工作区目录绑定、权限模式控制，实现真正的 AI Agent 交互能力

## Context Analysis

### Reference Code
- `reference/AINative/neuro-syntax-ide/src-tauri/src/lib.rs` — AINative 完整实现（6000+ 行），包含 `AgentRuntime` trait、`ClaudeCodeRuntime`、`CodexRuntime`、`GeminiHttpRuntime`、`RuntimeRegistry`、`RouterEngine`、`Claude CLI Bridge`（ReqAgent）
- 关键代码段：
  - L490-516: `AgentRuntime` trait 定义
  - L767-1120: `ClaudeCodeRuntime::execute()` — CLI 子进程管理 + stream-json 解析
  - L1459-1466: `create_default_registry()` — runtime 注册
  - L4397-4806: ReqAgent commands — `req_agent_start/send_message/stop/status`
  - L4898-4950: `runtime_execute` command — 通用 runtime 执行入口

### Related Documents
- `project-context.md` — SlockAI 架构定义，`runtime/` 模块规划
- `reference/AINative/module_5_ai_orchestration.md` — AI Orchestration 模块设计
- `reference/AINative/epic-neuro-syntax-ide-roadmap.md` — F7 feat-ai-agent-service 详细说明

### Related Features
- `feat-project-init` — 前置依赖，需要 Tauri V2 项目脚手架先搭建完成
- 未来 feature: Codex Runtime、Smart Router、Context Orchestrator

## Technical Solution

### 架构设计

```
src-tauri/src/
├── runtime/
│   ├── mod.rs              # AgentRuntime trait + ExecuteParams + StreamEvent
│   ├── claude.rs           # ClaudeCodeRuntime 实现
│   ├── registry.rs         # RuntimeRegistry (检测/注册/健康检查)
│   └── commands.rs         # Tauri commands (invoke handlers)
├── context/
│   └── ... (后续 feature)
├── storage/
│   └── ... (后续 feature)
└── lib.rs                  # 注册 commands + AppState
```

### AgentRuntime Trait

```rust
pub trait AgentRuntime: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn runtime_type(&self) -> &str;  // "cli" | "http"
    fn capabilities(&self) -> Vec<AgentCapability>;
    fn install_hint(&self) -> String;
    fn detect(&self) -> Result<Option<(String, String)>, String>;
    fn health_check(&self) -> AgentRuntimeStatus;
    fn info(&self) -> AgentRuntimeInfo;
    fn is_ready(&self) -> bool;
    fn execute(&self, params: ExecuteParams) -> Result<Receiver<StreamEvent>, String>;
}
```

### Claude Code CLI 调用模式

```bash
claude \
  --print \
  --output-format stream-json \
  --verbose \
  --resume <session-id> \
  --append-system-prompt "<system-prompt>" \
  --add-dir <workspace-path> \
  --permission-mode acceptEdits \
  --allowedTools "Read Write Glob Grep Bash Edit" \
  -- "<user-message>"
```

### stream-json 输出格式解析

```json
// assistant 消息 (--verbose 模式)
{"type":"assistant","message":{"content":[{"type":"text","text":"..."}]},"session_id":"..."}

// result 消息 (完成标记)
{"type":"result","subtype":"success","result":"...","session_id":"..."}

// system 消息
{"type":"system","subtype":"init","session_id":"..."}
```

### Tauri IPC 事件流

```
Rust execute() → stdout 逐行读取 → 解析 JSON →
  app_handle.emit("agent://chunk", StreamEvent) →
    前端 listen("agent://chunk") → 实时渲染
```

### 会话管理

- 每次消息调用创建独立子进程（per-message process model）
- 使用 `--resume <session-id>` 保持对话上下文
- session_id 由 Rust 端生成 UUID
- 不维护长驻进程，避免状态管理复杂性

### 安全设计

- API Key 通过 OS Keyring 存储（`keyring` crate）
- 前端永远无法获取 API Key 明文
- CLI 参数中不传递敏感信息

### AppState 扩展

```rust
struct AppState {
    agent_runtime_registry: Mutex<RuntimeRegistry>,
    agent_session: Mutex<AgentSessionState>,  // session_id + 当前进程引用
    // ... 其他状态
}
```

### Tauri Commands

| Command | 说明 |
|---------|------|
| `scan_agent_runtimes` | 检测所有已注册 runtime 的可用性 |
| `list_agent_runtimes` | 列出所有 runtime（使用缓存） |
| `runtime_execute` | 在指定 runtime 上执行消息，流式返回 |
| `runtime_session_start` | 创建新会话，返回 session_id |
| `runtime_session_stop` | 停止当前会话，清理进程 |

### 前端 Hook

```typescript
// useAgentRuntime.ts
interface AgentRuntimeState {
  runtimes: AgentRuntimeInfo[];
  scanning: boolean;
  scan: () => Promise<void>;
  execute: (runtimeId: string, message: string, sessionId?: string) => void;
  // ...
}
```

## Acceptance Criteria (Gherkin)

### User Story
作为 SlockAI 开发者，我需要 Agent Runtime 层能够检测和调用 Claude Code CLI，并将流式响应实时推送到前端，以便后续实现 @Agent 触发器机制。

### Scenario 1: Runtime 自动检测
```gherkin
Given Claude Code CLI 已安装在本机 (claude --version 可用)
When 调用 scan_agent_runtimes
Then 返回的 runtimes 列表中包含 claude-code runtime
And 其 status 为 "available"
And 包含 version 和 install_path 信息
```

### Scenario 2: CLI 不可用时降级
```gherkin
Given Claude Code CLI 未安装
When 调用 scan_agent_runtimes
Then 返回的 runtimes 列表中包含 claude-code runtime
And 其 status 为 "not-installed"
And 包含 install_hint: "npm install -g @anthropic-ai/claude-code"
```

### Scenario 3: 流式消息执行
```gherkin
Given Claude Code CLI 可用
And 已创建一个 session
When 调用 runtime_execute 发送 "hello" 消息
Then 前端通过 listen("agent://chunk") 收到多条 StreamEvent
And 最后一条 StreamEvent 的 is_done 为 true
And 消息中包含 session_id
```

### Scenario 4: 会话恢复
```gherkin
Given 已有一个 session_id
When 使用该 session_id 再次调用 runtime_execute 发送新消息
Then Claude Code CLI 使用 --resume 恢复上下文
And 响应中能引用之前的对话内容
```

### General Checklist
- [ ] AgentRuntime trait 可被多个 runtime 实现
- [ ] ClaudeCodeRuntime 通过 CLI 子进程调用
- [ ] stream-json 格式正确解析（assistant/result/system 三种消息类型）
- [ ] 前端通过 Tauri Event 实时收到流式响应
- [ ] 会话 ID 正确传递和恢复
- [ ] CLI 不可用时提供明确的错误提示和安装指引
- [ ] 使用 OS Keyring 保护 API Key

## Merge Record

- **Completed**: 2026-04-08T14:30:00+08:00
- **Merged Branch**: feature/feat-claude-runtime
- **Merge Commit**: 7ccc7b8
- **Archive Tag**: feat-claude-runtime-20260408
- **Conflicts**: None
- **Verification**: passed (4/4 scenarios)
- **Stats**: 1 commit, 11 files changed, 1273 insertions
