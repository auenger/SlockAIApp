# Tasks: feat-a2a-adapter

## Task Breakdown

### 1. CLI Adapter Trait (`cli_adapter.rs`)
- [ ] 定义 `CliA2AAdapter` trait（execute, cancel, get_status）
- [ ] 定义 `AdapterConfig` 结构体（workspace, system_prompt, timeout, env_vars）
- [ ] 定义 `AdapterState` 共享状态结构体（task_map, active_handles）

### 2. Claude Code Adapter (`claude_adapter.rs`)
- [ ] 实现 `ClaudeCodeA2AAdapter` 包装 `ClaudeCodeRuntime::execute()`
- [ ] 将 A2A SendMessage 映射为 ExecuteParams 并调用 claude CLI
- [ ] 将 CLI StreamEvent 流通过 SSE 回传给 A2A Client
- [ ] 处理 session_id 透传（--resume 参数）
- [ ] 处理 workspace 目录设置
- [ ] 错误映射（CLI 错误 → A2A Error）

### 3. Codex Adapter (`codex_adapter.rs`)
- [ ] 实现 `CodexA2AAdapter` 包装 `CodexRuntime::execute()`
- [ ] 同样的消息映射和流式回传逻辑
- [ ] Codex 特有的参数处理差异

### 4. A2A Server Handler 注册 (`server.rs` 扩展)
- [ ] 实现 Task CRUD handlers（CreateTask, GetTask, ListTasks, CancelTask）
- [ ] 实现 SendMessage handler（接收消息 → 分发到 adapter）
- [ ] 实现 StreamMessage handler（SSE 流式返回 adapter 输出）
- [ ] 实现 GetAgentCard handler（从 Agent 配置生成）
- [ ] 请求验证和错误处理中间件

### 5. AgentManager 集成
- [ ] AgentManager 启动 Agent 时创建对应的 A2A Server 实例
- [ ] AgentManager 停止 Agent 时关闭 A2A Server
- [ ] Agent 配置中增加 a2a_endpoint 字段（auto-generated 或手动指定）
- [ ] Agent 间的 @mention 触发走 A2A 协议而非直接 CLI 调用

### 6. Unix Socket / TCP 选择逻辑
- [ ] 默认使用 Unix socket（~/.agentszone/sock/）
- [ ] 配置项支持切换到 TCP localhost
- [ ] Socket 文件清理（进程退出时删除）
- [ ] 连接池管理

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-14 | Feature created | Initial task breakdown |
