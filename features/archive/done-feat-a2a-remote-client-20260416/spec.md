# Feature: feat-a2a-remote-client 远程 A2A Client + 连接管理（Connection-Centric）

## Basic Information
- **ID**: feat-a2a-remote-client
- **Name**: 远程 A2A Client + 连接管理（Connection-Centric 模型）
- **Priority**: 75
- **Size**: L
- **Dependencies**: [feat-a2a-transport]
- **Parent**: feat-a2a-runtime
- **Children**: []
- **Created**: 2026-04-14

## Merge Record
- **Completed**: 2026-04-16
- **Branch**: feature/feat-a2a-remote-client
- **Merge Commit**: 33cdbab
- **Archive Tag**: feat-a2a-remote-client-20260416
- **Conflicts**: none
- **Verification**: passed (73/77 tasks, 202/202 tests, 6 Gherkin scenarios analyzed)
- **Stats**: 18 files changed, 1674 insertions, 10 deletions

## Description

基于 **Connection-Centric 模型** 实现远程 Agent 的完整支持。核心思想：**Agent 不再绑定到本地 CLI 二进制，而是绑定到一个"连接端点"（可能是本地进程，也可能是远程 URL）。前端和上层业务逻辑完全不感知这种差异。**

本 Feature 是"远程 Claude Code / 远程 Agent"方案的**客户端部分**——与 P2（Adapter，服务端部分）配合使用。

### 解决什么问题？

当前执行链路的瓶颈：
```
Agent { runtime_type: "claude_code" }
  → RuntimeRegistry.find("claude-code")
  → which claude → 本地二进制路径
  → Command::new("claude").spawn()     ← 硬依赖本地进程
```

**`runtime_type` 只回答了"用什么运行"，没有回答"在哪里运行"。**

本 Feature 引入 `connection_mode` 字段，使 Agent 可以指向远程端点：
```
Agent {
  runtime_type: "claude_code",        // 标识能力类型（用什么）
  connection_mode: Remote {           // 标识连接方式（在哪里）
    connection_id: "conn-dev-server"
  }
}
  → RemoteConnectionManager.get("conn-dev-server")
  → endpoint_url = "https://dev-server:8443/a2a"
  → A2AHttpClient.send_message(...)   ← 标准 HTTP 协议
```

### 4 种场景覆盖

| 场景 | 描述 | 需要什么 | 本 Feature 覆盖 |
|------|------|---------|----------------|
| **S1: Local CLI** | `claude` 在本机 PATH | 已有代码 | 不变，兼容 |
| **S2: Remote A2A Adapter** | 远端机器运行 P2 Adapter 包装 Claude Code | **P2 + P3** | ✅ Client 部分 |
| **S3: Cloud SaaS** | 第三方 SaaS 提供 A2A 兼容 API | **P3** | ✅ 全部 |
| **S4: SSH Tunnel** | SSH 到远端执行 `claude` | 可选子功能 | ⚠️ 可选 |

**核心目标**：
- 新增 `remote_connections` 表和 `RemoteConnectionManager`
- 扩展 Agent 数据模型：增加 `connection_mode` + `remote_connection_id` 字段
- 实现 `RemoteA2ARuntime`（实现 `AgentRuntime` trait，通过 A2A HTTP 与远端通信）
- 认证层：API Key Bearer Token（存 Keyring）+ TLS 配置
- 前端 UI：Settings > Remote Connections 管理面板
- Agent 创建/编辑时支持选择 remote connection
- 远程 Agent 在 @mention 选择器中可见，对话体验与本地一致

**不做的事**：
- 不实现 Push Notification 回调服务端（P4）
- 不做复杂的负载均衡或多路复用
- 不修改核心 Channel/Thread 数据模型

## User Value Points

### V1: Connection-Centric 数据模型落地 ⭐ 核心
用户价值：系统原生区分本地/远程 Agent，数据模型清晰。
- SQLite 新增 `remote_connections` 表
- `agents` 表扩展 `connection_mode` + `remote_connection_id` 字段
- `AgentIdentity` Rust struct 扩展 `connection_mode: ConnectionMode` 字段
- 前端 TypeScript 类型同步扩展（AgentSummary, CreateAgentRequest 等）
- IDENTITY.md 文件格式扩展支持 `Connection Mode` 字段

### V2: 远程 Endpoint 配置与管理
用户价值：可以通过 GUI 管理 A2A 端点连接。
- Settings → Remote Connections 面板
- CRUD 操作（添加/编辑/删除远程 endpoint）
- 支持 HTTPS 端点（TLS 验证或 skip-cert 模式）
- 测试连接按钮（调用 GetAgentCard 验证连通性）
- 连接状态指示灯（Online=绿 / Offline=灰 / Error=红）

### V3: 认证与安全
用户价值：远程连接有基本安全保障。
- API Key Bearer Token 认证（存储在 Keyring，复用 keyring.rs）
- AuthType 枚举：None / ApiKey / OAuth2（预留）
- TLS 配置（verify_cert 开关，默认开启）
- Token 输入框 masked 显示，永不明文记录日志

### V4: 远程 Agent 对话集成 — 前端无感知 ⭐ 关键体验
用户价值：在 Channel 中使用远程 Agent 就像本地一样自然。
- 选择 remote Agent 发送消息时走 `RemoteA2ARuntime::execute()`
- 流式响应通过 SSE → bridge.to_stream_event() → 前端渲染
- **前端收到的仍然是 StreamEvent 格式——完全无感知**
- 错误处理友好提示（网络断开、认证失败、超时）
- @mention 选择器同时展示本地和远程 Agent，远程有特殊标识

## Context Analysis

### Reference Code（需修改的文件）

#### 后端需修改：
| 文件 | 当前状态 | 本 Feature 改动 |
|------|---------|---------------|
| `src-tauri/src/storage/migrations/V001__initial.sql` | agents 表无 connection_mode 字段 | **新增 migration** V00x: ALTER agents + CREATE remote_connections |
| `src-tauri/src/workspace/identity.rs` | AgentIdentity 无 connection_mode | **新增字段** `connection_mode: ConnectionMode` (Default::Local) |
| `src-tauri/src/workspace/manager.rs` | AgentManager 无 remote 概念 | **扩展** execute() 根据 connection_mode 分流 |
| `src-tauri/src/runtime/mod.rs` | AgentRuntime trait | **不变**（RemoteA2ARuntime 实现同一 trait） |
| `src-tauri/src/runtime/registry.rs` | RuntimeRegistry | **扩展** 支持注册 remote runtime |
| `src-tauri/src/storage/keyring.rs` | API Key 管理 | **复用** 存储远程连接 auth token |

#### 后端需新增：
| 文件 | 说明 |
|------|------|
| `src-tauri/src/runtime/a2a/remote.rs` | RemoteConnectionManager 核心 |
| `src-tauri/src/runtime/a2a/auth.rs` | 认证模块 |
| `src-tauri/src/runtime/a2a/remote_runtime.rs` | RemoteA2ARuntime (impl AgentRuntime trait) |
| `src-tauri/src/commands/remote_connection.rs` | IPC Commands |

#### 前端需修改：
| 文件 | 改动 |
|------|------|
| `src/types.ts` | AgentSummary / CreateAgentRequest 增加 connection_mode, remote_connection_id |
| `src/lib/ipc.ts` | 新增 remote_* IPC 函数 |

#### 前端需新增：
| 文件 | 说明 |
|------|------|
| `src/components/settings/RemoteConnectionsPanel.tsx` | 远程连接管理面板 |
| `src/lib/useRemoteConnections.ts` | Hook |

### Related Features
- **feat-a2a-transport** ⬅️ 直接依赖 — ConnectionMode/RemoteConnection 类型定义来自此处
- **feat-a2a-adapter** ➡️ 并行 — Adapter 是本 Feature 的"对端"（服务端）
- **feat-apikey-management-ui** ✅ 已完成 — Keyring 管理经验可复用
- **feat-agent-create-ui** ✅ 已完成 — Agent 创建 UI 需扩展 connection_mode 选择

## Technical Solution

<!-- 待实现时填充 -->

### 数据模型设计

```
┌─────────────────────────────────────────────────────────────┐
│  新增表: remote_connections                                  │
│                                                              │
│  id                  TEXT  PK  (UUID 或自定义 ID)             │
│  name                TEXT  NOT NULL  "我的开发服务器"         │
│  endpoint_url        TEXT  NOT NULL  "https://host:8443/a2a" │
│  auth_type           TEXT  NOT NULL  "none"|"api_key"|"oauth2"│
│  status              TEXT  NOT NULL  "unknown"                │
│  cached_agent_card   TEXT            JSON (AgentCard 缓存)    │
│  last_health_check_at TEXT                                    │
│  created_at          TEXT  NOT NULL                           │
│  updated_at          TEXT  NOT NULL                           │
│                                                              │
│  INDEX: idx_remote_connections_status (status)               │
│  INDEX: idx_remote_connections_name   (name)                 │
└──────────────────────────────┬──────────────────────────────┘
                               │ 1 : N
                               ▼
┌─────────────────────────────────────────────────────────────┐
│  扩展表: agents (ALTER TABLE ADD COLUMN)                     │
│                                                              │
│  ... 现有字段 ...                                            │
│  connection_mode      TEXT  NOT NULL DEFAULT 'local'         │
│                        -- "local" | "remote"                  │
│  remote_connection_id TEXT  FK → remote_connections(id)       │
│                        -- 仅当 connection_mode='remote' 时有值│
│                                                              │
│  CONSTRAINT: IF connection_mode='remote' THEN                │
│    remote_connection_id IS NOT NULL                          │
│    AND EXISTS (SELECT 1 FROM remote_connections               │
│                WHERE id = agents.remote_connection_id)        │
└─────────────────────────────────────────────────────────────┘
```

### Migration 设计

```sql
-- V00x__remote_connections.sql
-- 新增远程连接表 + 扩展 agents 表支持远程模式

CREATE TABLE IF NOT EXISTS remote_connections (
    id                      TEXT PRIMARY KEY,
    name                    TEXT NOT NULL,
    endpoint_url            TEXT NOT NULL,
    auth_type               TEXT NOT NULL DEFAULT 'none',
    status                  TEXT NOT NULL DEFAULT 'unknown',
    cached_agent_card       TEXT,           -- JSON blob
    last_health_check_at    TEXT,
    created_at              TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at              TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 扩展 agents 表
ALTER TABLE ADD COLUMN connection_mode      TEXT NOT NULL DEFAULT 'local';
ALTER TABLE ADD COLUMN remote_connection_id TEXT;

-- 索引
CREATE INDEX IF NOT EXISTS idx_remote_connections_status ON remote_connections(status);
CREATE INDEX IF NOT EXISTS idx_agents_connection_mode ON agents(connection_mode);
```

### Rust Struct 设计

```rust
// === AgentIdentity 扩展 ===
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentIdentity {
    // ... 现有字段 ...
    pub runtime_type: RuntimeType,
    // ─── 新增 ───
    /// 连接模式：Local（默认）或 Remote
    #[serde(default)]
    pub connection_mode: ConnectionMode,       // 来自 P1 types.rs
}

// === RemoteConnectionManager ===
pub struct RemoteConnectionManager {
    db: Arc<SqlitePool>,
    connections: HashMap<String, RemoteConnection>, // 缓存
    http_clients: HashMap<String, reqwest::Client>, // 按 endpoint 复用
}

impl RemoteConnectionManager {
    /// CRUD
    pub async fn create(&self, conn: CreateRemoteConnection) -> Result<RemoteConnection, String>;
    pub async fn list(&self) -> Vec<RemoteConnection>;
    pub async fn get(&self, id: &str) -> Option<RemoteConnection>;
    pub async fn update(&self, id: &str, update: UpdateRemoteConnection) -> Result<RemoteConnection, String>;
    pub async fn delete(&self, id: &str) -> Result<(), String>;

    /// 健康检查：GET {endpoint}/agent-card
    pub async fn health_check(&self, id: &str) -> Result<AgentCard, String>;

    /// 获取认证后的 HTTP client（自动注入 token）
    pub fn http_client(&self, id: &str) -> Option<&reqwest::Client>;
}

// === RemoteA2ARuntime: 实现 AgentRuntime trait ===
pub struct RemoteA2ARuntime {
    connection_id: String,
    manager: Arc<RemoteConnectionManager>,
}

impl AgentRuntime for RemoteA2ARuntime {
    fn id(&self) -> &str { "remote-a2a" }
    fn name(&self) -> &str { "Remote A2A Agent" }

    fn execute(&self, params: ExecuteParams) -> Result<Receiver<StreamEvent>, String> {
        // 1. 从 manager 获取 endpoint_url + auth token
        // 2. 通过 A2A HttpClient 创建 Task
        // 3. SendMessage
        // 4. StreamMessage (SSE)
        // 5. 将 SSE events bridge 为 StreamEvent → 返回 Receiver
    }

    // ... 其他 trait 方法 ...
}
```

### 执行路径分流逻辑

```rust
// AgentManager::execute_agent() 中的分流伪代码:
fn execute_agent(&self, agent_id: &str, message: &str) -> Result<Receiver<StreamEvent>, String> {
    let agent = self.agents.get(agent_id).ok_or("not found")?;

    match &agent.identity.connection_mode {
        ConnectionMode::Local => {
            // ===== 现有路径不变 =====
            let runtime = self.registry.get_runtime_instance(agent.identity.runtime_type.runtime_id())?;
            runtime.execute(ExecuteParams { message: message.to_string(), .. })
        }

        ConnectionMode::Remote { connection_id } => {
            // ===== 新增远程路径 =====
            let remote_runtime = RemoteA2ARuntime {
                connection_id: connection_id.clone(),
                manager: self.remote_conn_manager.clone(),
            };
            remote_runtime.execute(ExecuteParams { message: message.to_string(), .. })
        }
    }
}
```

### 前端 TypeScript 类型演进

```typescript
// === 新增类型 ===
export type ConnectionMode = "local" | "remote";
export type AuthType = "none" | "api_key" | "oauth2";
export type ConnectionStatus = "online" | "offline" | "error" | "unknown";

export interface RemoteConnection {
  id: string;
  name: string;
  endpoint_url: string;
  auth_type: AuthType;
  status: ConnectionStatus;
  agent_card?: AgentCard | null;  // 缓存的 AgentCard
  last_health_check_at?: string | null;
  created_at: string;
  updated_at: string;
}

// === 扩展现有类型 ===
export interface AgentSummary {
  // ... 现有字段 ...
  runtime_type: RuntimeType;
  // ─── 新增 ───
  connection_mode: ConnectionMode;        // 默认 "local"
  remote_connection_id?: string;         // remote 时必填
}

export interface CreateAgentRequest {
  // ... 现有字段 ...
  runtime_type?: RuntimeType;
  // ─── 新增 ───
  connection_mode?: ConnectionMode;       // 默认 "local"
  remote_connection_id?: string;         // remote 时必填
}
```

### UX 流程：添加一个远程 Agent

```
Step 1: 创建/配置 Remote Connection
┌─────────────────────────────────────────────┐
│  Settings → Remote Connections → [+ 添加]    │
│                                             │
│  名称:   [我的开发服务器          ]          │
│  URL:    [https://dev-server:8443/a2a]       │
│  认证:   ○ 无  ● API Key  ○ OAuth2          │
│  Key:    [sk-***....xyz          ] 🔒        │
│                                             │
│  [测试连接]  [保存]                           │
└─────────────────────────────────────────────┘
                │ 测试成功 → GET /agent-card → 缓存 AgentCard
                │          → status = Online 🟢
                ▼
Step 2: 创建 Agent 并绑定到该 Connection
┌─────────────────────────────────────────────┐
│  Create Agent                               │
│                                             │
│  Name:    [Remote Code Reviewer    ]        │
│  Runtime: [Claude Code  ▼]  (标识能力类型)  │
│  连接模式: ● 本地  ○ 远程                   │
│                                           │
│  ○ 远程时显示:                              │
│  端点: [我的开发服务器 ▼]  🟢 online        │
│  (从 AgentCard 自动填充 name/capabilities)  │
│                                             │
│  [创建]                                      │
└─────────────────────────────────────────────┘
                │
                ▼
Step 3: 使用 — 与本地 Agent 无差别
  Channel 中 @RemoteCodeReviewer → 发送消息
  → connection_mode=Remote → RemoteA2ARuntime.execute()
  → A2A HTTPS POST → 远端 A2A Server (P2 Adapter)
  → claude CLI 执行 → SSE stream back
  → bridge.to_stream_event() → 前端渲染 ✅
```

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望能够连接到远程机器上运行的 A2A 兼容 Agent（无论是通过我们的 Adapter 包装的 Claude Code，还是第三方 SaaS 服务），
以便在本地 AgentsZone 中使用远程算力或专用 Agent 能力，
且使用体验与本地 Agent 完全一致。

### Scenarios

#### Scenario 1: 数据模型 —— ConnectionMode 正确持久化
```gherkin
Given 数据库已执行 remote_connections migration
When 创建一个 Agent 并设置 connection_mode="remote", remote_connection_id="conn-1"
Then agents 表中 connection_mode = "remote"
And remote_connection_id = "conn-1"
And IDENTITY.md 文件包含 "- **Connection Mode**: remote" 行
When 重新加载 AgentManager
Then Agent 的 identity.connection_mode == Remote { connection_id: "conn-1" }
```

#### Scenario 2: 添加并测试远程 Endpoint
```gherkin
Given 用户在 Settings > Remote Connections 页面
When 用户输入:
  - name="我的开发服务器"
  - endpoint_url="https://dev-server:8443/a2a"
  - auth_type="api_key", token="sk-xxx"
And 点击"保存"
Then 远程连接信息写入 remote_connections 表
And auth token 加密存储到 Keyring (key="remote_conn_conn-1")
And 连接列表中显示新条目, status="unknown"

When 用户点击"测试连接"
Then 系统发送 GET {endpoint}/agent-card (带 Bearer Token)
If 成功则:
  - 缓存返回的 AgentCard 到 cached_agent_card 字段
  - 更新 status="online", last_health_check_at=now()
  - UI 显示 AgentCard 信息（name, capabilities, version）
If 失败则:
  - 更新 status="error"
  - UI 显示具体错误原因（网络/认证 401/超时）
```

#### Scenario 3: 通过远程 Agent 对话 —— 核心端到端场景
```gherkin
Given 一个已配置的远程连接 (id="conn-1", status="online")
And 一个 Agent 绑定到此连接 (connection_mode=remote, conn_id="conn-1")
When 用户在 Channel 中 @mention 此 Agent 并发送 "帮我重构这个函数"
Then AgentManager.execute_agent() 检测到 connection_mode=Remote
And 创建 RemoteA2ARuntime 实例
And 通过 A2A HttpClient 执行:
  1. POST {endpoint}/tasks → 创建 Task (status=SUBMITTED)
  2. POST {endpoint}/tasks/{id}/messages → SendMessage
  3. GET {endpoint}/tasks/{id}/messages/stream → SSE StreamMessage
And SSE 事件通过 bridge 转换为 StreamEvent
And 前端实时渲染流式响应（与本地 Agent 渲染完全一致）
And 响应完成后 Task 标记为 COMPLETED
```

#### Scenario 4: 远程连接异常处理
```gherkin
Given 正在与远程 Agent 对话中 (SSE streaming)
When 网络突然断开
Then SSE 连接报错 → 转换为 StreamEvent { error: "连接中断", is_done: true }
And 前端显示"连接中断"错误提示
And 提供"重试"按钮
When 用户点击重试
Then 重新建立 A2A 连接（创建新 Task 或 Resume 旧 Task）
And 未完成的上下文保留在用户消息中
```

#### Scenario 5: 远程 Agent 在 @mention Selector 中可见
```gherkin
Given 存在:
  - 2 个本地 Agent (connection_mode=local)
  - 1 个远程 Agent (connection_mode=remote, conn status=online)
When 用户在 Channel 输入框输入 "@"
Then @mention 弹出选择器同时显示 3 个 Agent
And 远程 Agent 有特殊图标标识（如云朵 icon + "Remote" badge）
And 显示连接状态指示灯（🟢 online / 🔴 offline / 🟡 unknown）
When 用户选择远程 Agent
Then 该 Agent 的消息走 RemoteA2ARuntime 路径
```

#### Scenario 6: 回归 —— 本地 Agent 行为不变
```gherkin
Given 一个现有 Agent (未设置 connection_mode, 默认 Local)
When 用户与此 Agent 对话
Then 执行路径完全走原有 RuntimeRegistry → ClaudeCodeRuntime/CodexRuntime
And 不经过任何 A2A HTTP 层
And 性能和行为与 Feature 开发前完全一致
```

### General Checklist

#### 数据层
- [ ] Migration SQL 编写并通过（remote_connections 表 + agents 扩展字段）
- [ ] RemoteConnection CRUD 存储操作
- [ ] AgentIdentity connection_mode 字段的序列化/反序列化（含 IDENTITY.md 格式）
- [ ] AgentSummary / IdentitySummary Rust struct 同步扩展
- [ ] 向后兼容：旧 Agent 的 connection_mode 默认为 Local

#### 后端逻辑
- [ ] RemoteConnectionManager 完整实现（CRUD + health check + client pool）
- [ ] Auth 模块（API Key 注入、Keyring 存储/读取、TLS 配置）
- [ ] RemoteA2ARuntime 实现 AgentRuntime trait（execute 分流核心）
- [ ] AgentManager.execute_agent() 的 connection_mode 分流逻辑
- [ ] RuntimeRegistry 支持注册 RemoteA2ARuntime
- [ ] IPC Commands 全部实现（create/list/update/delete/test/health）

#### 前端
- [ ] TypeScript 类型同步扩展（ConnectionMode, RemoteConnection, AgentSummary 等）
- [ ] IPC 函数封装（src/lib/ipc.ts）
- [ ] RemoteConnectionsPanel 组件（CRUD + 测试 + 状态展示）
- [ ] Agent 创建/编辑 UI 扩展（connection_mode 选择器 + remote connection 下拉）
- [ ] @mention Selector 增强（远程 Agent 展示 + 状态标识）
- [ ] 远程 Agent 对话 UI（流式渲染 + 错误提示 + 重试）

#### 安全
- [ ] Auth tokens 存储在 Keyring，不在 DB 明文保存
- [ ] Token 不出现在日志或错误消息中
- [ ] HTTPS 强制（可配置 skip-cert 用于开发环境）
- [ ] endpoint URL 输入验证（防止 SSRF）

#### 质量
- [ ] cargo build + npm run build 全部通过
- [ ] S6 回归场景验证通过
- [ ] 所有新增代码遵循项目约定
