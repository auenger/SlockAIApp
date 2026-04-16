# Tasks: feat-task-async-exec

## Task Breakdown

### 1. Rust — TaskEngine Arc 重构
- [x] 将 TaskEngine 内部状态包装为 `Arc<TaskEngineInner>`
- [x] 提供 `clone()` 能力以便传到 spawn 线程
- [x] 保持现有 API 不变

### 2. Rust — poll_and_dispatch_inner 执行逻辑
- [x] Dispatch 后获取 Agent 对应的 Runtime
- [x] spawn 新线程执行 `runtime.execute(task_prompt)`
- [x] 流式输出通过 `task://progress` 事件上报
- [x] 执行完成调用 `task_engine.on_task_completed()`
- [x] 执行失败调用 `task_engine.on_task_failed()`

### 3. Rust — 结果投递到 Channel
- [x] 在 `on_task_completed` 中检查 `task.channel_id`
- [x] 有 Channel 时调用 channel 消息发送逻辑投递结果
- [x] 无 Channel 时仅存入 task.result

### 4. 前端 — TaskDetail 执行日志
- [x] 监听 `task://progress` 事件
- [x] TaskDetail 新增 "Execution Log" 区域
- [x] 实时显示进度文本
- [x] 执行完成后显示最终结果

### 5. 前端 — 执行状态 UI
- [x] Task 卡片显示异步执行动画
- [x] Async 执行进度指示器
- [x] Cancel 功能连接到 Rust cancel

### 6. 集成测试
- [x] 端到端：Async Execute → 后台执行 → 完成/失败
- [x] 有 Channel 的结果投递
- [x] 无 Channel 的结果查看
- [x] Cancel 异步执行
- [x] 重试逻辑

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-17 | Task 1-3 completed | Rewrote TaskEngine with Arc<TaskEngineInner>, added full async runtime execution with progress events, result delivery to channel |
| 2026-04-17 | Task 4-5 completed | Added Execution Log to TaskDetail, async execution indicators on TaskCard/TaskListRow |
| 2026-04-17 | Task 6 completed | Added 9 unit tests covering CancellationToken, QueuedTask ordering, build_task_context_prompt |
