# Tasks: feat-a2a-adapter

## Task Breakdown

### 1. CLI Adapter Trait (`cli_adapter.rs`)
- [x] 定义 `CliA2AAdapter` trait（execute, cancel, get_status）
- [x] 定义 `AdapterConfig` 结构体（workspace, system_prompt, timeout, env_vars）
- [x] 定义 `AdapterState` 共享状态结构体（task_map, active_handles）

### 2. Claude Code Adapter (`claude_adapter.rs`)
- [x] 实现 `ClaudeCodeA2AAdapter` 包装 `ClaudeCodeRuntime::execute()`
- [x] 将 A2A SendMessage 映射为 ExecuteParams 并调用 claude CLI
- [x] 将 CLI StreamEvent 流通过 SSE 回传给 A2A Client
- [x] 处理 session_id 透传（--resume 参数）
- [x] 处理 workspace 目录设置
- [x] 错误映射（CLI 错误 → A2A Error）

### 3. Codex Adapter (`codex_adapter.rs`)
- [x] 实现 `CodexA2AAdapter` 包装 `CodexRuntime::execute()`
- [x] 同样的消息映射和流式回传逻辑
- [x] Codex 特有的参数处理差异

### 4. A2A Server Handler 注册 (`server.rs` 扩展)
- [x] 实现 Task CRUD handlers（CreateTask, GetTask, ListTasks, CancelTask）
- [x] 实现 SendMessage handler（接收消息 → 分发到 adapter）
- [x] 实现 StreamMessage handler（SSE 流式返回 adapter 输出）
- [x] 实现 GetAgentCard handler（从 Agent 配置生成）
- [x] 请求验证和错误处理中间件

### 5. AgentManager 集成
- [ ] AgentManager 启动 Agent 时创建对应的 A2A Server 实例
- [ ] AgentManager 停止 Agent 时关闭 A2A Server
- [ ] Agent 配置中增加 a2a_endpoint 字段（auto-generated 或手动指定）
- [ ] Agent 间的 @mention 触发走 A2A 协议而非直接 CLI 调用

### 6. Unix Socket / TCP 选择逻辑
- [x] 默认使用 Unix socket（~/.agentszone/sock/）
- [x] 配置项支持切换到 TCP localhost
- [x] Socket 文件清理（进程退出时删除） — SocketGuard RAII guard
- [x] 连接池管理 — ConnectionPool with bounded per-endpoint limits + idle eviction

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-14 | Feature created | Initial task breakdown |
| 2026-04-15 | Tasks 1-4, 6(partial) implemented | Core adapter + handler + listener. 26 tests passing. cargo build + cargo test pass. |
| 2026-04-15 | Task 6 completed | SocketGuard RAII cleanup + ConnectionPool with bounded limits, idle eviction, auto-release. Task 5 (AgentManager integration) deferred per spec. |

