# Tasks: feat-a2a-remote-client

## Task Breakdown

### 1. 数据层：Migration + 存储操作
- [x] 编写 migration SQL（V005__remote_connections.sql）：
  - [x] CREATE TABLE remote_connections（id, name, endpoint_url, auth_type, status, cached_agent_card, last_health_check_at, timestamps）
  - [x] ALTER TABLE agents ADD COLUMN connection_mode TEXT DEFAULT 'local'
  - [x] ALTER TABLE agents ADD COLUMN remote_connection_id TEXT
  - [x] CREATE INDEX idx_remote_connections_status
  - [x] CREATE INDEX idx_agents_connection_mode
- [x] 实现 RemoteConnection 存储 CRUD（db_helpers.rs）
- [x] 向后兼容：旧 Agent 的 connection_mode 默认为 "local"

### 2. 后端：AgentIdentity / AgentSummary 扩展
- [x] `identity.rs`: AgentIdentity 新增 `connection_mode: ConnectionMode` 字段（Default::Local）
- [x] `identity.rs`: IdentitySummary 同步新增字段
- [x] `identity.rs`: IDENTITY.md 序列化/反序列化支持 `Connection Mode` 行
- [x] `identity.rs`: parse_identity_content() 解析 `- **Connection Mode**: remote` 格式
- [x] `manager.rs`: Agent struct 同步携带 connection_mode 信息

### 3. 后端：认证模块
- [x] API Key Bearer Token 注入 reqwest header（在 RemoteA2ARuntime + RemoteConnectionManager 中实现）
- [x] Token 存储到 Keyring（key = `"remote_conn_{conn_id}"`, 复用 keyring.rs）
- [x] Token 从 Keyring 读取并验证存在性
- [x] TLS 配置 — danger_accept_invalid_certs（开发环境）

### 4. 后端：RemoteConnectionManager 核心 (`remote.rs`)
- [x] `RemoteConnectionManager` — 无状态设计，接受 `&Connection` 参数
- [x] `create(db, name, url, auth_type)` → 写入 DB + 返回 RemoteConnection
- [x] `list(db)` → 查询 DB 所有连接
- [x] `get(db, id)` → 查询单个
- [x] `update(db, id, patch)` → 更新 DB
- [x] `delete(db, id)` → 删除 DB + 清理 Keyring token
- [x] `health_check(db, id)` → GET {endpoint}/agent-card → 更新 status + 缓存 AgentCard
- [x] `health_check_all(db)` → 批量健康检查
- [x] `store_auth_token(id, token)` → 存储到 Keyring
- [x] `get_auth_token(id)` → 从 Keyring 读取

### 5. 后端：RemoteA2ARuntime — 实现 AgentRuntime trait
- [x] `RemoteA2ARuntime` 结构体（持有 RemoteConnection）
- [x] 实现 `id()` → connection.id
- [x] 实现 `name()` → connection.name
- [x] 实现 `runtime_category()` → "http"
- [x] 实现 `typed_runtime_type()` → RuntimeType::Custom("remote-a2a")
- [x] 实现 `capabilities()` → [Streaming, ToolUse]
- [x] 实现 `detect()` → 基于 connection.status
- [x] 实现 `health_check()` → 基于 connection status
- [x] 实现 `info()` → 完整 AgentRuntimeInfo
- [x] 实现 `is_ready()` → connection.status == Online
- [x] **实现 `execute(params)` — 核心方法**：
  - [x] 获取 auth token + build A2AHttpClient
  - [x] 生成 task_id
  - [x] 通过 A2ATransport::stream_message() 发送 SSE 请求
  - [x] 通过 mpsc::channel 返回 Receiver<StreamEvent>
  - [x] 错误处理：网络错误、认证失败 → StreamEvent { error, is_done }

### 6. 后端：AgentManager 分流逻辑
- [x] `AgentManager.create_agent()` 增加 `connection_mode` 参数
- [x] `AgentSummary` 新增 `connection_mode` 字段
- [x] `create_agent_internal()` 写入 identity.connection_mode
- [x] `create_agent` IPC command 传递 connection_mode

### 7. 后端：IPC Commands (`commands/remote_connection.rs`)
- [x] `remote_connection_create(name, endpoint_url, auth_type, api_key?)`
- [x] `remote_connection_list()`
- [x] `remote_connection_update(id, patch)`
- [x] `remote_connection_delete(id)`
- [x] `remote_connection_test(id)`
- [x] `remote_connection_health_all()`
- [x] `remote_connection_get_agent_card(id)`
- [x] 在 `lib.rs` 中注册所有 commands

### 8. 前端：数据类型 + IPC 封装
- [x] TypeScript 类型定义：
  - [x] `ConnectionMode`, `RemoteAuthType`, `RemoteConnectionStatus`
  - [x] `RemoteConnectionInfo` interface
  - [x] `RemoteAgentCard` interface
  - [x] `TestConnectionResult` interface
  - [x] `CreateRemoteConnectionRequest` / `UpdateRemoteConnectionRequest`
  - [x] 扩展 `AgentSummary`（+connection_mode）
  - [x] 扩展 `IdentitySummary`（+connection_mode）
  - [x] 扩展 `CreateAgentRequest`（+connection_mode）
- [x] IPC 函数封装（src/lib/ipc.ts）：
  - [x] `remoteConnectionCreate(...)`
  - [x] `remoteConnectionList()`
  - [x] `remoteConnectionUpdate(...)`
  - [x] `remoteConnectionDelete(id)`
  - [x] `remoteConnectionTest(id)`
  - [x] `remoteConnectionHealthAll()`
  - [x] `remoteConnectionGetAgentCard(id)`

### 9. 前端：UI - RemoteConnectionsPanel
- [x] 连接列表视图（卡片式布局，每卡显示 name, url, status badge）
- [x] 添加连接表单（Name, URL, Auth Type, API Key masked input）
- [x] 编辑连接（复用添加表单，预填数据）
- [x] 删除连接确认对话框
- [x] 测试连接按钮 → loading state → 结果展示（成功/失败）
- [x] 状态指示灯（Online=绿 / Unknown=黄 / Error=红 / Offline=灰）
- [x] 批量健康检查按钮（Check All）
- [x] useRemoteConnections hook

### 10. 前端：Agent 创建/编辑 UI 扩展
- [ ] Create Agent 表单新增"连接模式"选择器（预留，当前通过 API 支持）
- [ ] Edit Agent 表单同样支持修改 connection_mode

### 11. 前端：@mention Selector 增强
- [ ] @mention 弹出列表同时展示本地和远程 Agent
- [ ] 远程 Agent 特殊视觉标识（云朵 icon + 状态点）

### 12. 前端：对话体验一致性
- [x] 远程 Agent 消息发送 UI 与本地完全一致（同一 Channel input 组件）
- [x] 流式渲染逻辑无需改动（收到的都是 StreamEvent）
- [x] 错误处理通过 StreamEvent.error 传递

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-14 | Feature created | Initial task breakdown |
| 2026-04-14 | Major rewrite | Connection-Centric model: data model, execution path split, 4 scenarios |
| 2026-04-16 | Implementation | Backend: migration, db helpers, identity, manager, RemoteConnectionManager, RemoteA2ARuntime, IPC commands |
| 2026-04-16 | Implementation | Frontend: types, IPC, RemoteConnectionsPanel, useRemoteConnections hook |
| 2026-04-16 | Bug fixes | Fixed compilation errors: ConnectionMode arg passing, lifetime issues, unused imports, test updates |
| 2026-04-16 | Verified | cargo check + npx tsc --noEmit pass, 202 Rust tests pass |
