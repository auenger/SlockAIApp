# Tasks: feat-a2a-transport

## Task Breakdown

### 1. A2A 协议类型定义 (`types.rs` — A2A 协议部分)
- [x] 定义 `TaskStatus` 枚举（SUBMITTED / WORKING / COMPLETED / FAILED / CANCELED / REJECTED / INPUT_REQUIRED / AUTH_REQUIRED）
- [x] 定义 `Task` 结构体（id, status, session_id, messages, artifacts, metadata）
- [x] 定义 `Message` 结构体（role, parts, context, metadata）
- [x] 定义 `Part` enum（TextPart / FilePart / DataPart / InlineDataPart）
- [x] 定义 `Artifact` 结构体（id, name, description, parts, created_at）
- [x] 定义 `AgentCard` 结构体（name, description, capabilities, endpoint, auth）
- [x] 定义 `A2AError` 结构体（code, message, details）
- [x] 定义 `SendMessageRequest` / `GetTaskRequest` / `CancelTaskRequest` 等 Request/Response 类型
- [x] 所有类型实现 Serialize/Deserialize + Debug
- [x] 编写类型序列化/反序列化单元测试

### 1.5. 连接模型类型定义 (`types.rs` — Connection-Centric 部分) ⭐ 新增
- [x] 定义 `ConnectionMode` 枚举（Local / Remote { connection_id: String }），实现 Default=Local
- [x] 定义 `RemoteConnection` 结构体（id, name, endpoint_url, auth_type, status, cached_agent_card, last_health_check_at, created_at, updated_at）
- [x] 定义 `AuthType` 枚举（None / ApiKey / OAuth2）
- [x] 定义 `ConnectionStatus` 枚举（Online / Offline / Error / Unknown）
- [x] 定义 `PushNotificationConfig` 结构体（预留 P4 使用：url, token, events 过滤列表）
- [x] 所有连接模型类型实现 Serialize/Deserialize + Debug + PartialEq
- [x] 编写连接模型类型的序列化 round-trip 单元测试
- [x] 编写 ConnectionMode JSON tag 变体测试（确保 Local vs Remote 的 serde 正确）

### 2. A2A Transport Trait + HTTP 实现 (`transport.rs`, `client.rs`)
- [x] 定义 `A2ATransport` trait（send_message, get_task, cancel_task, list_tasks, stream_message, get_agent_card）
- [x] 实现 `A2AHttpClient` 基于 reqwest
- [x] 实现 JSON-RPC 请求封装（method, params, id, jsonrpc version）
- [x] 实现错误处理链（网络错误 → A2AError → Result<T>）
- [x] 编写 Client 单元测试（mock server via wiremock 或类似）

### 3. SSE Streaming 支持 (`streaming.rs`)
- [x] 实现 `stream_message()` 使用 reqwest SSE 能力
- [x] 解析 SSE data 行为 A2A Message
- [x] 处理 SSE 连接管理（重连、超时、取消）
- [x] 将 SSE 事件流转换为 `Receiver<StreamEvent>` 兼容格式
- [x] 编写 Streaming 单元测试

### 4. A2A Server 骨架 (`server.rs`)
- [x] 定义 `A2AServer` trait 或 struct（handler 注册模式）
- [x] 实现 HTTP route 骨架（POST /tasks, GET /tasks/{id}, POST /tasks/{id}/messages 等）
- [x] 实现 GET /agent-card 端点
- [x] Server 可配置 port 和 bind address
- [x] 编写 Server 骨架集成测试

### 5. 现有 Runtime 桥接 (`bridge.rs`)
- [x] 实现 `stream_event_to_a2a_message()` 转换函数
- [x] 实现 `a2a_message_to_stream_event()` 反向转换函数
- [x] 处理所有已知 msg_type（assistant, user, system, result, raw, stderr, tool_use, tool_result）
- [x] content_blocks ↔ Artifact 映射逻辑
- [x] TaskStatus ↔ 执行状态映射
- [x] 编写桥接转换单元测试

### 6. 模块集成 (`mod.rs`)
- [x] 创建 `runtime/a2a/` 模块并导出所有子模块
- [x] 在 `runtime/mod.rs` 中添加 `pub mod a2a`
- [x] 确保 `cargo build` 通过无 warning
- [x] 更新 Cargo.toml 如需新增依赖（reqwest 特性等）

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-14 | Feature created | Initial task breakdown |
| 2026-04-14 | Updated with Connection-Centric types (ConnectionMode, RemoteConnection, AuthType, ConnectionStatus) |
| 2026-04-16 | All tasks implemented | 6 modules: types, transport, streaming, server, bridge, mod. 78 tests pass. |
