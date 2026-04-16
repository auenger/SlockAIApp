# Tasks: feat-task-async-exec

## Task Breakdown

### 1. Rust — TaskEngine Arc 重构
- [ ] 将 TaskEngine 内部状态包装为 `Arc<TaskEngineInner>`
- [ ] 提供 `clone()` 能力以便传到 spawn 线程
- [ ] 保持现有 API 不变

### 2. Rust — poll_and_dispatch_inner 执行逻辑
- [ ] Dispatch 后获取 Agent 对应的 Runtime
- [ ] spawn 新线程执行 `runtime.execute(task_prompt)`
- [ ] 流式输出通过 `task://progress` 事件上报
- [ ] 执行完成调用 `task_engine.on_task_completed()`
- [ ] 执行失败调用 `task_engine.on_task_failed()`

### 3. Rust — 结果投递到 Channel
- [ ] 在 `on_task_completed` 中检查 `task.channel_id`
- [ ] 有 Channel 时调用 channel 消息发送逻辑投递结果
- [ ] 无 Channel 时仅存入 task.result

### 4. 前端 — TaskDetail 执行日志
- [ ] 监听 `task://progress` 事件
- [ ] TaskDetail 新增 "Execution Log" 区域
- [ ] 实时显示进度文本
- [ ] 执行完成后显示最终结果

### 5. 前端 — 执行状态 UI
- [ ] Task 卡片显示异步执行动画
- [ ] Async 执行进度指示器
- [ ] Cancel 功能连接到 Rust cancel

### 6. 集成测试
- [ ] 端到端：Async Execute → 后台执行 → 完成/失败
- [ ] 有 Channel 的结果投递
- [ ] 无 Channel 的结果查看
- [ ] Cancel 异步执行
- [ ] 重试逻辑

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
