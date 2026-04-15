# Feature: feat-a2a-adapter 本地 Runtime → A2A Server Adapter

## Basic Information
- **ID**: feat-a2a-adapter
- **Name**: 本地 Runtime → A2A Server Adapter（CLI 包装为 A2A 端点）
- **Priority**: 78
- **Size**: M
- **Dependencies**: [feat-a2a-transport]
- **Parent**: feat-a2a-runtime
- **Children**: []
- **Created**: 2026-04-14

## Description

将现有的 CLI-based Runtime（ClaudeCodeRuntime / CodexRuntime）包装为 A2A Server，使本地 Agent 通过标准 A2A 协议对外暴露能力。

**这个 Adapter 是"远程 Claude Code"方案的核心组件**——它运行在远端机器上，将那台机器上的 `claude` CLI 暴露为标准 A2A 端点。本机通过 Phase 3 的 Remote Client 连接到此端点。

**核心目标**：
- 复用现有 `claude.rs` / `codex.rs` 的 CLI 调用逻辑，外层套 A2A HTTP 接口
- 本地 Agent 启动时自动注册为 A2A Server（Unix socket 或 localhost port）
- 其他组件通过 A2A 协议与本地 Agent 通信，而非直接调用 CLI
- 支持 Unix Socket 模式实现零网络开销的本地通信

**在 Connection-Centric 模型中的角色**：

```
本 Feature 解决的是 S2 场景的"服务端"部分：

  远端机器 (运行本 Adapter):
    Claude Code CLI ──→ A2A Adapter (本 Feature) ──→ A2A HTTP Endpoint
                                                    ↑
  本机 (运行 P3 Remote Client):                      │
    Agent { connection_mode: Remote } ──→ A2A HttpClient ─┘
```

**不做的事**：
- 不实现远程连接客户端（那是 P3 的事）
- 不修改前端 UI
- 不改变 Agent 创建/编辑流程（但 P3 会改）

## User Value Points

### V1: 本地 Agent A2A 化
用户价值：本地运行的 Claude Code / Codex Agent 变成标准化的 A2A 端点，任何支持 A2A 的客户端都能与之交互。
- ClaudeCodeRuntimeAdapter：包装 claude.rs execute() 为 A2A Task 执行
- CodexRuntimeAdapter：包装 codex.rs execute() 为 A2A Task 执行
- 每个 Agent 实例对应一个 A2A Server 实例

**关键使用场景**：在远端开发服务器上部署此 Adapter → 该服务器上的 Claude Code 变成 A2A 端点 → 本机 AgentsZone 可以远程使用。

### V2: 零开销本地通信
用户价值：本地 Agent 间通信不经过 TCP/IP 栈，性能无损。
- Unix Socket 绑定模式（默认本地）
- TCP localhost 回退模式
- 自动选择最优传输方式

### V3: AgentCard 动态生成
用户价值：每个 Agent 能力自描述，便于发现和匹配。
- 从 Agent 配置（runtime_type, capabilities, workspace）生成 AgentCard
- 支持自定义 capability 声明
- GET /agent-card 返回实时状态

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/claude.rs:117-473` — `execute()` 方法是核心被包装逻辑
- `src-tauri/src/runtime/a2a/types.rs` — A2A 类型定义 + 连接模型（依赖 P1）
- `src-tauri/src/runtime/a2a/server.rs` — A2A Server 骨架（依赖 P1）
- `src-tauri/src/workspace/identity.rs` — AgentIdentity（含 runtime_type 字段）
- `src-tauri/src/workspace/manager.rs` — AgentManager 管理多 Agent生命周期

### Related Features
- **feat-a2a-transport** ⬅️ 直接依赖 — 必须先完成类型定义和 Transport 层（含 ConnectionMode 等类型）
- **feat-agent-runtime-model** ✅ 已完成 — Runtime trait 泛化基础
- **feat-claude-runtime** ✅ 已完成 — Claude Code CLI 调用逻辑
- **feat-a2a-remote-client** ➡️ 并行依赖 — 本 Adapter 是 Remote Client 的"对端"

## Technical Solution

### Architecture

```
src-tauri/src/runtime/a2a/
├── adapter/
│   ├── mod.rs           # Adapter 模块导出
│   ├── cli_adapter.rs   # CliA2AAdapter trait + AdapterConfig + AdapterState
│   ├── claude_adapter.rs # ClaudeCodeAdapter — wraps ClaudeCodeRuntime
│   ├── codex_adapter.rs  # CodexAdapter — wraps CodexRuntime
│   └── handler.rs       # AdapterServer + ListenerConfig + TCP handler + AgentCard generation
```

### Key Design Decisions

1. **Adapter trait pattern**: `CliA2AAdapter` trait provides a clean abstraction over CLI runtimes.
   Each adapter wraps an existing `AgentRuntime::execute()` and maps A2A Task lifecycle to CLI process lifecycle.

2. **Arc<dyn CliA2AAdapter>**: The adapter is shared between the AdapterServer and its closures via Arc,
   enabling multiple handlers to reference the same adapter instance safely.

3. **Status tracking via spawn_status_tracker**: A background thread reads StreamEvents from the CLI runtime,
   updates the shared AdapterState (task status + session_id), and forwards events to the caller.

4. **Non-invasive wrapping**: The adapter layer does NOT modify claude.rs or codex.rs. It wraps their
   existing `execute()` method, preserving all existing behavior.

5. **Unix socket + TCP**: ListenerConfig supports both modes with auto-socket-path generation.
   TCP listener provides a simple HTTP server for integration testing and remote deployment.

### Task 5 (AgentManager Integration) — deferred

AgentManager integration is a deeper concern that touches the Tauri app lifecycle:
- Starting/stopping A2A Server instances when agents activate/deactivate
- Adding `a2a_endpoint` field to agent configuration
- Routing @mention triggers through A2A protocol

This is intentionally deferred to keep the current scope focused on the adapter infrastructure.

### 调用链路（本地模式 + 远端部署模式）

```
┌─────────────────────── 本机/远端机器 ───────────────────────┐
│                                                               │
│  前端 IPC → Command → AgentManager.execute()                  │
│                                  │                            │
│                    ┌─────────────┴─────────────┐              │
│                    │  A2A Transport             │ (P1 提供)    │
│                    │  (local unix sock / TCP)    │              │
│                    ↓                             │              │
│              ┌──────────────┐                    │              │
│              │ A2A Server   │ (P1 server.rs 骨架) │              │
│              │ Handler      │                    │              │
│              ↓    ↓    ↓                         │              │
│        SendMessage  GetTask  CancelTask           │              │
│              ↓                                 │              │
│     ┌──────────────────┐                        │              │
│     │ CLI Adapter      │ (本 Feature 核心)       │              │
│     │ claude.execute() │ (已有代码, 不改动)       │              │
│     └──────────────────┘                        │              │
│                                                   │              │
│  当部署在远端时:                                   │              │
│    本机的 P3 Remote Client ──HTTP(SSE)──→ 此 A2A Server         │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### 远程 Claude Code 部署拓扑

```
┌──────────────────┐         ┌──────────────────────────┐
│  你的 Mac        │  HTTPS  │  开发服务器 (Linux)       │
│  (AgentsZone)    │ ←────→ │                          │
│                  │  A2A    │  ┌────────────────────┐  │
│  Remote Agent    │  协议   │  │ A2A Adapter        │  │
│  (P3 Client)     │         │  │ (本 Feature)       │  │
│                  │         │  │       ↓            │  │
│                  │         │  │  claude CLI        │  │
│                  │         │  │  (已安装)           │  │
│                  │         │  └────────────────────┘  │
└──────────────────┘         └──────────────────────────┘
```

## Acceptance Criteria (Gherkin)

### User Story
作为系统架构的一部分，我希望将现有 CLI-based runtime 包装为 A2A Server，
以便所有 Agent 交互都通过标准化协议进行，同时保留已有的 CLI 调用稳定性。
当此 Adapter 部署在远端机器上时，它使该机器上的 Claude Code 成为本机可用的远程 A2A 端点。

### Scenarios

#### Scenario 1: Claude Code 作为 A2A Server 启动
```gherkin
Given 一个配置了 Claude Code runtime 的 Agent
When AgentManager 启动该 Agent
Then 在 Unix socket (或 localhost port) 上启动 A2A Server
And Server 监听 POST /tasks/{id}/messages 等标准端点
And 调用底层 claude.rs execute() 处理请求
```

#### Scenario 2: 通过 A2A 发送消息给本地 Agent
```gherkin
Given 本地 Claude Code Agent 的 A2A Server 运行中
When 通过 A2A Client 发送 SendMessage
Then 消息传递到 CLI Adapter
And Adapter 调用 claude --print "message"
And CLI 输出通过 bridge 转换为 A2A Message 返回
```

#### Scenario 2.5: 远端部署 —— 从本机通过 A2A 访问 ⭐ 核心场景
```gherkin
Given 远端开发服务器上运行着 A2A Adapter (包装了该服务器上的 claude CLI)
And 本机 AgentsZone 配置了一个 RemoteConnection 指向该服务器 (P3)
When 用户在本机 Channel 中 @mention 了绑定到此远端的 Agent
Then 消息通过 A2A HTTPS 发送到远端 A2A Server
And 远端 Adapter 接收消息 → 调用本地 claude CLI → 流式返回结果
And 本机前端渲染流式响应（与本地 Agent 体验一致）
```

#### Scenario 3: Unix Socket 本地通信
```gherkin
Given A2A Server 配置为 local 模式
When Server 启动
Then 绑定到 Unix socket (~/.agentszone/sock/{agent_id}.sock)
And Client 通过同一 socket 连接
And 无 TCP/IP 开销
```

#### Scenario 4: AgentCard 自描述
```gherkin
Given 一个 Agent 配置了 runtime=claude_code, capabilities=[streaming, tool_use]
When 请求 GET /agent-card
Then 返回包含 name, description, capabilities, endpoint 的 JSON
And capabilities 与 Agent 配置一致
```

#### Scenario 5: Task 生命周期映射
```gherkin
Given 通过 A2A 创建了一个 Task
When CLI 开始执行
Then Task status = WORKING
When CLI 返回 result
Then Task status = COMPLETED (成功) 或 FAILED (错误)
When 用户取消
Then Task status = CANCELED
```

### General Checklist
- [ ] ClaudeCodeAdapter 包装 claude.rs execute() 成功
- [ ] CodexAdapter 包装 codex.rs execute() 成功
- [ ] Unix Socket 模式正常工作
- [ ] TCP localhost fallback 正常工作
- [ ] AgentCard 正确生成并可通过 HTTP 获取
- [ ] Task 状态机正确反映 CLI 执行状态
- [ ] 远端部署模式下 HTTPS/A2A 通信正常（需配合 P3 测试）
- [ ] 现有功能不受影响（回归测试通过）
- [ ] cargo build + cargo test 全部通过
