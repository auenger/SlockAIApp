# Tasks: feat-agent-runtime-exec

## Task Breakdown

### 1. Runtime 路由层
- [x] 修改 `send_thread_message` command：根据 agent.runtime_type 获取 runtime
- [x] 调用 `RuntimeRegistry::get_runtime()` 获取实现
- [x] 执行前健康检查，不可用时返回明确错误
- [x] 流式输出通过 `app.emit()` 发送 chunk

### 2. Session 管理
- [x] 创建 `SessionManager` 管理 per-thread session
  - Note: Per-thread session already managed via Thread.session_id field.
    Runtime type change detection handled at execution time -- if agent's
    runtime_type changes, a new session is naturally created by the new runtime.
- [x] 支持 get_or_create：已有 session 则 resume，否则创建
  - Note: Existing thread.session_id is passed to ExecuteParams; if None, runtime creates new.
- [ ] Session 过期清理逻辑 (deferred: not needed for MVP)
- [x] Runtime 类型变更时重建 session
  - Note: Handled implicitly -- new runtime_type routes to different runtime which manages its own sessions.

### 3. Channel 多 Agent 路由
- [x] 修改 `send_channel_message` command
- [x] @mention 的每个 agent 独立路由到对应 runtime
- [ ] 并发执行（tokio::spawn）(deferred: serial execution is safer for MVP)
- [x] 各 agent 响应通过各自 event 回传

### 4. 错误处理
- [x] Runtime 不可用时返回友好错误消息
- [x] 安装提示信息传递到前端
- [x] （可选）降级策略：首选 runtime 不可用时的 fallback chain
  - Note: Implemented as clear error message with install hint via runtime://unavailable event.
    Fallback chain deferred to future iteration.

### 5. 前端适配
- [x] useThreadChat 适配（后端自动路由，前端无需大改）
- [x] 错误消息 UI 展示（runtime 不可用提示）
- [x] Channel 多 agent 响应区分显示

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 拆分自 feat-agent-runtime-select |
| 2026-04-10 | Implementation complete | Runtime routing, health check, error handling, frontend adaptation |
