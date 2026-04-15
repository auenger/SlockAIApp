# Tasks: feat-task-execution

## Task List

### Task 1: Rust — TaskEngine 模块框架

- [x] 新建 src-tauri/src/task_engine/ 目录
- [x] 新建 src-tauri/src/task_engine/mod.rs — TaskEngine struct 定义
- [x] CancellationToken 实现 (AtomicBool)
- [x] ActiveTask / QueuedTask 结构体
- [x] TaskEngine::new() 初始化
- [x] 在 src-tauri/src/lib.rs 中注册 TaskEngine 为 Tauri managed state

### Task 2: Rust — 实时执行逻辑

- [x] TaskEngine::execute_realtime() — 注入 Task 上下文到 channel.rs send_message
- [x] agent_busy 按 (agent_id, channel_id) 粒度跟踪
- [x] 检查依赖 check_dependencies()
- [x] 更新 DB status → in_progress
- [x] 注入 extra_system_prompt (Task context: title + description + source summary)
- [x] 注册 on_complete 回调到 channel.rs

### Task 3: Rust — 异步执行逻辑

- [x] TaskEngine::enqueue() — 放入异步队列
- [x] 后台 poll 线程 (每 5 秒轮询)
- [x] poll_and_dispatch() — 找到空闲 Agent + 无依赖阻塞的 Task
- [x] AsyncTaskContext 构建 (workspace + task_prompt)
- [x] 创建后台 Thread 执行异步 Task
- [x] TaskEngine::start_poll_thread() 启动后台线程

### Task 4: Rust — 取消 + 重试机制

- [x] TaskEngine::cancel_task() — 实时: cancel token; 异步: 从队列移除
- [x] on_task_completed() — 更新 result + status + emit event
- [x] on_task_failed() — retry_count < MAX_RETRY -> 重新入队; 否则 -> failed
- [x] 错误重试逻辑 (MAX_RETRY=2)

### Task 5: Rust — Tauri Commands 集成

- [x] 扩展 commands/task.rs: execute_task command
- [x] 扩展 commands/task.rs: cancel_task command (已有，需与 TaskEngine 集成)
- [x] Tauri Events 推送: task://status-changed, task://progress, task://completed, task://failed, task://cancelled, task://retry
- [x] channel.rs 完成回调中调用 TaskEngine.on_task_completed/on_task_failed

### Task 6: TS — useTaskEngine Hook + IPC

- [x] src/lib/ipc.ts 添加 execute_task IPC 封装
- [x] src/lib/useTaskEngine.ts — 执行状态 hook
- [x] 监听 task://* 事件更新状态
- [x] executeTask / cancelTask 暴露给 UI

### Task 7: TS — 执行 UI 组件

- [x] 执行按钮组件 (Task 卡片/列表上的 play 按钮)
- [x] 进度条/执行中状态展示
- [x] 取消按钮 (执行中可取消)
- [x] 结果展示 (完成后显示 result)

## Progress Log

| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-15 | Feature started | 从 feat-agent-task-system 拆分，依赖 feat-task-data-model 已完成 |
| 2026-04-15 | All tasks implemented | Rust backend (Tasks 1-5) + TS frontend (Tasks 6-7) complete |
