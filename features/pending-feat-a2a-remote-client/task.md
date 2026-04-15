# Tasks: feat-a2a-remote-client

## Task Breakdown

### 1. 数据层：Migration + 存储操作
- [ ] 编写 migration SQL（V00x__remote_connections.sql）：
  - [ ] CREATE TABLE remote_connections（id, name, endpoint_url, auth_type, status, cached_agent_card, last_health_check_at, timestamps）
  - [ ] ALTER TABLE agents ADD COLUMN connection_mode TEXT DEFAULT 'local'
  - [ ] ALTER TABLE agents ADD COLUMN remote_connection_id TEXT
  - [ ] CREATE INDEX idx_remote_connections_status
  - [ ] CREATE INDEX idx_agents_connection_mode
- [ ] 实现 RemoteConnection 存储 CRUD（db.rs 或新建 remote_connection_store.rs）
- [ ] 向后兼容：旧 Agent 的 connection_mode 默认为 "local"

### 2. 后端：AgentIdentity / AgentSummary 扩展
- [ ] `identity.rs`: AgentIdentity 新增 `connection_mode: ConnectionMode` 字段（Default::Local）
- [ ] `identity.rs`: IdentitySummary 同步新增字段
- [ ] `identity.rs`: IDENTITY.md 序列化/反序列化支持 `Connection Mode` 行
- [ ] `identity.rs`: parse_identity_content() 解析 `- **Connection Mode**: remote` 格式
- [ ] `manager.rs`: Agent struct 同步携带 connection_mode 信息

### 3. 后端：认证模块 (`auth.rs`)
- [ ] 定义 `AuthConfig` 结构体（auth_type: AuthType, keyring_key: String）
- [ ] API Key Bearer Token 注入 reqwest header（`Authorization: Bearer {token}`）
- [ ] Token 存储到 Keyring（key = `"remote_conn_{conn_id}"`, 复用 keyring.rs）
- [ ] Token 从 Keyring 读取并验证存在性
- [ ] TLS 配置结构体（verify_cert: bool, ca_cert_path: Option<String>）
- [ ] OAuth2 预留接口（token refresh callback，暂不实现）

### 4. 后端：RemoteConnectionManager 核心 (`remote.rs`)
- [ ] `RemoteConnectionManager` 结构体（db 引用、连接缓存、HTTP client 池）
- [ ] `create(conn)` → 写入 DB + 返回 RemoteConnection
- [ ] `list()` → 查询 DB 所有连接
- [ ] `get(id)` → 查询单个（含缓存检查）
- [ ] `update(id, patch)` → 更新 DB + 刷新缓存
- [ ] `delete(id)` → 删除 DB + 清理缓存 + 清理 Keyring token
- [ ] `health_check(id)` → GET {endpoint}/agent-card → 更新 status + 缓存 AgentCard
- [ ] `health_check_all()` → 批量健康检查（并发）
- [ ] `http_client(id)` → 获取/创建带认证的 reqwest::Client（按 endpoint 复用）
- [ ] 定时健康检查（可选后台任务，默认关闭，可配置开启）

### 5. 后端：RemoteA2ARuntime — 实现 AgentRuntime trait ⭐ 核心组件
- [ ] `RemoteA2ARuntime` 结构体（connection_id, manager Arc）
- [ ] 实现 `id()` → "remote-a2a"
- [ ] 实现 `name()` → "Remote A2A Agent"
- [ ] 实现 `runtime_type()` → RuntimeType::Custom("remote-a2a") 或映射到实际类型
- [ ] 实现 `capabilities()` → 从缓存的 AgentCard 提取
- [ ] 实现 `detect()` → 检查远程连接是否 online
- [ ] 实现 `health_check()` → 基于 connection status
- [ ] **实现 `execute(params)` — 核心方法**：
  - [ ] 获取 RemoteConnection 和 HTTP client
  - [ ] POST {endpoint}/tasks 创建 A2A Task
  - [ ] POST {endpoint}/tasks/{task_id}/messages 发送用户消息
  - [ ] SSE GET {endpoint}/tasks/{task_id}/messages/stream 接收流式响应
  - [ ] 将 SSE events 通过 bridge (P1) 转换为 StreamEvent
  - [ ] 通过 mpsc::channel 返回 Receiver<StreamEvent>
  - [ ] 错误处理：网络错误、认证失败(401)、超时 → StreamEvent { error, is_done }
  - [ ] idle watchdog 复用现有模式
- [ ] 支持 session_id 透传（Resume 远程 Task）

### 6. 后端：AgentManager 分流逻辑
- [ ] `AgentManager` 新增 `remote_conn_manager: Arc<RemoteConnectionManager>` 字段
- [ ] `execute_agent()` 方法增加 connection_mode 分支判断：
  - [ ] Local → 现有路径不变（RuntimeRegistry → ClaudeCodeRuntime.execute()）
  - [ ] Remote → 新路径（RemoteA2ARuntime.execute()）
- [ ] `list_agents()` 返回值包含 connection_mode 信息
- [ ] 创建 Agent 时支持传入 connection_mode + remote_connection_id

### 7. 后端：IPC Commands (`commands/remote_connection.rs`)
- [ ] `remote_connection_create(name, endpoint_url, auth_type, api_key?)` → 创建连接
- [ ] `remote_connection_list()` → 列出所有连接（不含敏感 token）
- [ ] `remote_connection_update(id, patch)` → 更新连接配置
- [ ] `remote_connection_delete(id)` → 删除连接（级联清理关联 Agent？或标记 orphan）
- [ ] `remote_connection_test(id)` → 测试连接（返回 AgentCard 或 ErrorDetail）
- [ ] `remote_connection_health_all()` → 批量健康检查
- [ ] `remote_connection_get_agent_card(id)` → 获取缓存的 AgentCard
- [ ] 在 `lib.rs` 中注册所有 commands

### 8. 前端：数据类型 + IPC 封装
- [ ] TypeScript 类型定义：
  - [ ] `ConnectionMode`, `AuthType`, `ConnectionStatus`
  - [ ] `RemoteConnection` interface
  - [ ] 扩展 `AgentSummary`（+connection_mode, +remote_connection_id）
  - [ ] 扩展 `CreateAgentRequest`（+connection_mode, +remote_connection_id）
  - [ ] 扩展 `IdentitySummary`（+connection_mode）
- [ ] IPC 函数封装（src/lib/ipc.ts）：
  - [ ] `remoteConnectionCreate(...)`
  - [ ] `remoteConnectionList()`
  - [ ] `remoteConnectionUpdate(...)`
  - [ ] `remoteConnectionDelete(id)`
  - [ ] `remoteConnectionTest(id)`
  - [ ] `remoteConnectionHealthAll()`
- [ ] Tauri event listener（可选：监听 connection status 变更事件）

### 9. 前端：UI - RemoteConnectionsPanel
- [ ] Settings 页面中的 Remote Connections 子面板入口
- [ ] 连接列表视图（卡片式布局，每卡显示 name, url, status badge）
- [ ] 添加连接对话框/表单：
  - [ ] 名称输入
  - [ ] URL 输入（含格式验证）
  - [ ] 认证方式选择（None / API Key / OAuth2 disabled）
  - [ ] API Key 输入框（password 类型，masked 显示）
  - [ ] TLS 验证开关（高级选项）
- [ ] 编辑连接（复用添加表单，预填数据）
- [ ] 删除连接确认对话框
- [ ] 测试连接按钮 → loading state → 结果展示（成功=AgentCard info / 失败=error detail）
- [ ] 状态指示灯（🟢 Online / 🟡 Unknown / 🔴 Offline / 🔴 Error）

### 10. 前端：Agent 创建/编辑 UI 扩展
- [ ] Create Agent 表单新增"连接模式"选择器：
  - [ ] 单选：● 本地（默认） / ○ 远程
  - [ ] 选择"远程"时显示 Remote Connection 下拉选择框
  - [ ] 下拉数据从 remoteConnectionList() 获取
  - [ ] 仅显示 status=online 的连接（或全部 + 灰显离线的）
- [ ] Edit Agent 表单同样支持修改 connection_mode
- [ ] 连接模式变更时的联动提示（"切换到远程将使用 A2A 协议通信"等）

### 11. 前端：@mention Selector 增强
- [ ] @mention 弹出列表同时展示本地和远程 Agent
- [ ] 远程 Agent 特殊视觉标识：
  - [ ] 云朵 icon 或 "Remote" 文字 badge
  - [ ] 连接状态小圆点（🟢🔴🟡）
- [ ] 选择远程 Agent 后的提示（可选 tooltip 显示 endpoint URL）

### 12. 前端：对话体验一致性
- [ ] 远程 Agent 消息发送 UI 与本地完全一致（同一 Channel input 组件）
- [ ] 流式渲染逻辑无需改动（收到的都是 StreamEvent）
- [ ] 错误状态展示：
  - [ ] 连接失败 toast/notification
  - [ ] 认证失败 (401) 特殊提示："认证失败，请检查 API Key"
  - [ ] 超时提示："远程 Agent 响应超时"
  - [ ] 网络断开提示："网络连接中断" + 重试按钮
- [ ] 重试机制：重新调用 execute()（可能创建新 Task 或 resume）

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-14 | Feature created | Initial task breakdown |
| 2026-04-14 | Major rewrite with Connection-Centric model: data model changes, execution path split, 4 scenarios (S1-S4), full UX flow |
