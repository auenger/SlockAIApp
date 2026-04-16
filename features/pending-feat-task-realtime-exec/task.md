# Tasks: feat-task-realtime-exec

## Task Breakdown

### 1. 前端 — useTaskEngine 扩展
- [ ] 在 `useTaskEngine` 添加 `onRealtimeExecute` 回调参数
- [ ] 在 `task://execute-realtime` 监听中调用回调
- [ ] 保持 activeTasks 追踪逻辑不变

### 2. 前端 — TaskView 层连接
- [ ] 实现 `handleRealtimeExecute` 回调
- [ ] 调用 `sendChannelMessage(channel_id, task_prompt, userName)`
- [ ] 格式化 task_prompt：包含 Task 标题、描述、执行指令

### 3. Rust — Agent 响应完成自动回调
- [ ] 在 `send_channel_message` 的 agent 响应流程中检测 task 上下文
- [ ] Agent 完成回复后自动调用 `task_engine.on_task_completed(task_id, result)`
- [ ] Agent 执行出错时调用 `task_engine.on_task_failed(task_id, error)`

### 4. 前端 — 执行状态 UI
- [ ] Task 卡片显示 "Executing..." 状态
- [ ] Execute 按钮变为 Cancel 按钮
- [ ] TaskDetail 面板显示执行进度

### 5. 集成测试
- [ ] 端到端验证：Execute → Channel 消息 → Agent 回复 → Task 完成
- [ ] 验证 tool use 渲染正常
- [ ] 验证 cancel 功能正常

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
