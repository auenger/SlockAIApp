# Tasks: feat-remote-agent-chat
## Task Breakdown
### 1. RemoteRuntime 实现
- [x] 实现 `AgentRuntime` trait for `RemoteRuntime`
- [x] A2A 消息构建（JSON-RPC 格式）
- [x] 流式响应接收和转发
- [x] 超时处理和错误恢复
- [x] 修复 streaming.rs 消息未传递 bug

### 2. Runtime Registry 集成
- [x] `RuntimeRegistry` 根据 `connection_mode` 返回对应 runtime
- [x] 远程 runtime 实例管理（动态创建 RemoteA2ARuntime）
- [x] `resolve_runtime_for_agent` 辅助函数

### 3. Channel @mention 远程 Agent
- [x] Channel 执行引擎支持远程 agent 路由
- [x] 远程 agent 响应流式发送到前端
- [x] 多 agent 并行执行（本地+远程混合）

### 4. Thread 远程对话
- [x] Thread 命令支持远程 runtime 执行
- [x] 远程对话持久化到本地 JSONL

### 5. 错误处理
- [x] 连接断开时友好提示
- [x] 执行超时处理
- [x] A2A 协议错误转换

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-17 | Feature created | 等待 feat-remote-agent-ui 完成 |
| 2026-04-17 | Implementation complete | 5个任务全部完成，270个测试通过 |
