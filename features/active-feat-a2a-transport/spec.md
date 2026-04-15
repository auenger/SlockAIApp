# Feature: feat-a2a-transport A2A 协议类型定义 + Transport 基础设施

## Basic Information
- **ID**: feat-a2a-transport
- **Name**: A2A 协议类型定义 + Transport 基础设施
- **Priority**: 80
- **Size**: M
- **Dependencies**: []
- **Parent**: feat-a2a-runtime
- **Children**: []
- **Created**: 2026-04-14

## Description

为 AgentsZone 引入 Google A2A (Agent-to-Agent) 协议的 Rust 类型定义和 Transport 抽象层。这是整个 A2A Runtime 重构的基础设施——后续所有阶段都依赖于此模块提供的类型系统和通信原语。

**核心目标**：
- 定义完整的 A2A 协议数据模型（Task / Message / Artifact / AgentCard 等）
- 定义连接模型类型（ConnectionMode / RemoteConnection / AuthType）—— 为远程 Agent 支撑
- 实现 HTTP + JSON-RPC 的 Transport 层（Client + Server 骨架）
- 支持 SSE 流式传输（StreamMessage 操作）
- 提供与现有 `AgentRuntime` trait 的桥接接口

**不做的事**：
- 不替换现有的 CLI 调用逻辑
- 不实现远程连接管理或认证逻辑（Phase 3）
- 不修改前端 UI
- 不改变现有数据存储结构（但新增 `remote_connections` 表的 migration 骨架）

## 设计理念：Connection-Centric 模型

当前系统的瓶颈：`runtime_type` 只回答"用什么运行"，不回答"在哪里运行"。整个链路假设 runtime 始终是本地进程。

本 Phase 引入 **Connection-Centric** 数据模型的核心类型：

```
Agent 不再绑定到一个本地 CLI 二进制，而是绑定到一个"连接端点"：
  - Local  → 本地 CLI 进程（现有行为不变）
  - Remote → 远程 A2A 端点 URL（新能力）
```

### 4 种执行场景

| 场景 | 描述 | 对应 Phase |
|------|------|-----------|
| **S1: Local CLI** | `claude` 在本机 PATH 上，直接 spawn | 现有代码不变 |
| **S2: Remote A2A (Adapter)** | 远端机器运行我们的 A2A Adapter 包装 Claude Code | P2 Adapter + P3 Client |
| **S3: Cloud SaaS** | 第三方 SaaS 提供 A2A 兼容 Agent API | P3 Client |
| **S4: SSH Tunnel** | SSH 到远端执行 `claude`，流式传回 | P3 可选子功能 |

## User Value Points

### V1: 标准化协议类型系统
用户价值：拥有与业界标准对齐的数据模型，为后续远程 Agent 互操作打下基础。
- 完整的 A2A v1.0.0 类型定义（Rust struct + serde 序列化）
- Task 状态机枚举（SUBMITTED → WORKING → COMPLETED/FAILED/CANCELED...）
- Message / Artifact / Part 数据结构
- AgentCard 自描述能力声明

### V1.5: 连接模型类型系统 ⭐ 新增
用户价值：数据模型原生支持"本地 vs 远程"区分，为后续 Phase 打下基础。
- `ConnectionMode` 枚举：`Local` | `Remote { connection_id }`
- `RemoteConnection` 结构体：端点 URL、认证方式、状态、缓存的 AgentCard
- `AuthType` 枚举：`None` | `ApiKey` | `OAuth2`
- `ConnectionStatus` 枚举：`Online` | `Offline` | `Error` | `Unknown`
- 这些类型在此 Phase 定义但不在本 Phase 使用——供 P2/P3 消费

### V2: HTTP Transport 双向通信能力
用户价值：具备通过标准 HTTP 协议发送/接收 A2A 消息的能力。
- A2A Client：SendMessage / GetTask / CancelTask / ListTasks 等 RPC 调用
- A2A Server：HTTP handler 骨架，可注册 handler 回调
- SSE StreamMessage 支持（reqwest eventsource）
- 错误码标准化（A2A Error format）

### V3: 与现有 Runtime 桥接
用户价值：新 Transport 层可以无缝接入现有 AgentRuntime 体系。
- `A2ATransport` trait 抽象（支持 HTTP / gRPC / local 后续扩展）
- `StreamEvent` ↔ A2A `Message` 转换器
- Registry 支持注册 A2A transport 实例

## Context Analysis

### Reference Code
- `src-tauri/src/runtime/mod.rs` — 现有 AgentRuntime trait、StreamEvent、ExecuteParams、RuntimeType 定义
- `src-tauri/src/runtime/claude.rs:117-473` — Claude Code CLI 执行逻辑（将被 adapter 包装）
- `src-tauri/src/runtime/registry.rs` — RuntimeRegistry 注册中心
- `src-tauri/src/workspace/identity.rs` — AgentIdentity 结构体（需扩展 connection_mode 字段）
- `src-tauri/src/storage/migrations/V001__initial.sql` — agents 表 schema（需扩展 connection_mode, remote_connection_id）
- `src/types.ts` — 前端 AgentSummary / CreateAgentRequest 类型（需同步扩展）

### Related Documents
- [A2A Protocol Spec v1.0.0](https://github.com/google/A2A) — Google 官方协议规范
- CLAUDE.md — 项目架构约定

### Related Features
- **feat-task-data-model** ✅ 已完成 — Task 数据模型可与 A2A Task 映射
- **feat-agent-runtime-model** ✅ 已完成 — AgentRuntime trait 泛化
- **feat-agent-a2a-trigger** ✅ 已完成 — @{agent} 触发机制（将升级为 A2A 触发）

## Technical Solution

<!-- 待实现时填充 -->

### 架构概要

```
src-tauri/src/runtime/a2a/
├── mod.rs              # 模块导出
├── types.rs            # A2A 协议类型定义 + 连接模型类型
│   │
│   │  ── A2A 协议类型 ──
│   ├── Task            # 任务实体 + 状态机
│   ├── Message         # 消息（user/agent/system）
│   ├── Artifact        # 产出物（文件/代码/图表）
│   ├── Part            # 内容片段（text/image/data）
│   ├── AgentCard       # Agent 能力自描述
│   ├── Error           # A2A 标准错误码
│   │
│   │  ── 连接模型类型（Connection-Centric 核心）⭐ ──
│   ├── ConnectionMode      # enum { Local, Remote { connection_id } }
│   ├── RemoteConnection    # struct (id, name, endpoint_url, auth_type, status, ...)
│   ├── AuthType            # enum { None, ApiKey, OAuth2 }
│   ├── ConnectionStatus    # enum { Online, Offline, Error, Unknown }
│   └── PushNotificationConfig  # 预留：P4 使用
│
├── transport.rs        # A2ATransport trait + HTTP 实现
├── client.rs           # A2A HTTP Client (reqwest)
├── server.rs           # A2A HTTP Server (骨架)
├── streaming.rs        # SSE StreamMessage 支持
└── bridge.rs           # StreamEvent ↔ A2A Message 转换
```

### 数据模型演进（本 Phase 只定义类型，不改表结构）

```rust
// === 连接模式：Agent 的"在哪里运行"属性 ===
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionMode {
    /// 本地 CLI（现有行为不变）
    Local,
    /// 远程 A2A 端点
    Remote { connection_id: String },
}

impl Default for ConnectionMode {
    fn default() -> Self { Self::Local }
}

// === 远程连接：一个 A2A 端点的完整配置 ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConnection {
    pub id: String,
    pub name: String,                          // "我的开发服务器"
    pub endpoint_url: String,                   // "https://dev-server:8443/a2a"
    pub auth_type: AuthType,
    pub status: ConnectionStatus,
    pub cached_agent_card: Option<AgentCard>,   // TTL 缓存
    pub last_health_check_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// === 认证方式 ===
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    None,
    ApiKey,     // Bearer Token，存 Keyring
    OAuth2,     // 预留
}

// === 连接状态 ===
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionStatus {
    Online,
    Offline,
    Error,
    Unknown,
}
```

### 执行路径对比（类型层面）

```
┌──────────────────────────────────────────────────────────────┐
│                     统一入口                                  │
│  AgentManager.execute(agent_id, message)                      │
│       │                                                       │
│       ▼                                                       │
│  agent.connection_mode                                        │
│       │                                                       │
│  ┌────┴──────────────────────────────────┐                    │
│  │                                       │                    │
│  ▼                                       ▼                    │
│ Local                              Remote { conn_id }        │
│  │                                       │                    │
│  ▼                                       ▼                    │
│ RuntimeRegistry                       RemoteConnectionManager  │
│   .get(runtime_type)                     .get(conn_id)          │
│  │                                       │                    │
│  ▼                                       ▼                    │
│ ClaudeCodeRuntime.execute()          A2AHttpClient            │
│   → Command::new("claude").spawn()      .send_message(url)     │
│   → stdout stream-json                  POST /tasks/{id}/msgs  │
│   → BufReader → StreamEvent             → SSE stream           │
│  │                                       │                    │
│  ▼                                       ▼                    │
│  前端渲染（完全一致）                        bridge.to_stream_event() │
│                                       → 前端渲染（完全一致）    │
└──────────────────────────────────────────────────────────────┘
```

**关键洞察：前端完全无感知！两种路径输出的都是 `StreamEvent`。**

## Acceptance Criteria (Gherkin)

### User Story
作为开发者，我希望引入 A2A 协议的类型定义、Transport 基础设施和连接模型，
以便后续可以将现有 CLI-based runtime 迁移到标准化协议上，并支持远程 Agent。

### Scenarios

#### Scenario 1: A2A 类型定义完整性
```gherkin
Given A2A protocol types 模块已编译
When 我创建一个 Task 实例
Then 该 Task 包含 id, status(SUBMITTED), session_id, messages 列表
And status 可以从 SUBMITTED 变更为 WORKING
And 所有类型均可 serde 序列化/反序列化为合法 JSON
```

#### Scenario 1.5: 连接模型类型定义 ⭐ 新增
```gherkin
Given Connection-Centric 类型模块已编译
When 创建一个 ConnectionMode::Remote { connection_id: "conn-1".into() }
Then 可正确序列化为 JSON { "remote": { "connection_id": "conn-1" } }
When 创建一个 RemoteConnection { endpoint_url: "https://x:8443/a2a", auth_type: ApiKey }
Then 所有字段均可序列化/反序列化
And ConnectionStatus 默认值为 Unknown
And AuthType 默认值为 None
```

#### Scenario 2: A2A Client 发送消息
```gherkin
Given 一个 A2A HTTP Client 实例指向 localhost:8080
When 我调用 client.send_message(task_id, message)
Then Client 发送 POST /tasks/{task_id}/messages 请求
And 返回包含 updatedTask 的 A2A 响应
And 网络错误被转换为 A2A Error 类型
```

#### Scenario 3: SSE 流式接收
```gherkin
Given 一个远程 A2A Server 正在流式输出任务消息
When 我调用 client.stream_message(task_id)
Then 通过 SSE 接收增量 Message 事件
And 每个 event 可解析为 A2A Message
And 连接断开时返回明确错误
```

#### Scenario 4: 现有 StreamEvent 桥接
```gherkin
Given 一个来自 claude.rs 的 StreamEvent { text, msg_type: "assistant", content_blocks }
When 调用 bridge::stream_event_to_a2a_message(event)
Then 返回 A2A Message { role: "agent", parts: [...] }
And content_blocks 中的 tool_use/tool_result 映射为 Artifact
```

#### Scenario 5: AgentCard 自描述
```gherkin
Given 一个本地 Claude Code Agent
When 生成其 AgentCard
Then 包含 capabilities: ["streaming", "tool_use", "sessions"]
And 包含 supported_operations: ["sendMessage", "streamMessage", ...]
And 可序列化后通过 HTTP GET /agent-card 返回
```

### General Checklist
- [ ] 所有 A2A 协议类型定义通过 `cargo test` 编译和单元测试
- [ ] 所有连接模型类型（ConnectionMode, RemoteConnection, AuthType, ConnectionStatus）定义完成并可序列化
- [ ] HTTP Client 可成功发送请求并解析响应（需 mock server 测试）
- [ ] SSE streaming 在单元测试中验证事件解析
- [ ] Bridge 转换器覆盖所有已知 StreamEvent msg_type
- [ ] 无新增外部依赖导致的编译问题（仅 reqwest + tokio + serde）
- [ ] 文档注释覆盖所有 public API
