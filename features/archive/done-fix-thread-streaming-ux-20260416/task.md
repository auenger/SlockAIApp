# Tasks: fix-thread-streaming-ux

## Task Breakdown

### 1. Backend — Thread Agent Start 事件
- [x] 在 `thread.rs send_message` 中 runtime 执行前 emit `agent://thread-agent-start` 事件
- [x] 包含 thread_id, agent_id, runtime_id, runtime_name

### 2. useThreadChat Hook 增强
- [x] 新增 `contentBlocks: ContentBlock[]` 状态
- [x] 新增 `statusMessage?: string` 状态
- [x] 新增 `isDone: boolean` 状态
- [x] 监听 `agent://thread-agent-start` 事件设置 statusMessage
- [x] 处理 `agent://chunk` 事件中的 content_blocks 和 system 事件
- [x] `send()` 中重置新增状态
- [x] `clearActive()` 中清理新增状态
- [x] 在 hook 返回值中导出新增状态

### 3. ThreadPanel.tsx UI 对齐
- [x] 替换 Thinking 指示器：去掉 `animate-pulse` + 灰色占位条，改为 "Thinking" + 跳动灰色圆点 + statusMessage
- [x] 增强 Streaming 指示器：添加 ContentBlock 卡片渲染
- [x] 添加 Done 状态显示

### 4. MainContent.tsx Thread 模式 UI 对齐
- [x] 替换 Thread 模式 Thinking 指示器为 Channel 风格
- [x] 增强 Thread 模式 Streaming 指示器，添加 ContentBlock 卡片
- [x] 添加 Done 状态显示
- [x] 从 useThreadChat 接收新增的 contentBlocks 和 statusMessage

### 5. App.tsx — Props 透传
- [x] 从 useThreadChat 解构 contentBlocks, statusMessage, isDone
- [x] 透传至 MainContent 和 ThreadPanel

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-16 | Feature created | 初始任务拆解 |
| 2026-04-16 | Task 1 completed | Backend thread-agent-start event |
| 2026-04-16 | Task 2 completed | useThreadChat hook with contentBlocks, statusMessage, isDone |
| 2026-04-16 | Task 3 completed | ThreadPanel UI aligned with Channel style |
| 2026-04-16 | Task 4 completed | MainContent Thread mode UI aligned |
| 2026-04-16 | Task 5 completed | App.tsx props wired |
| 2026-04-16 | Build verified | TypeScript + Vite + Rust cargo check all pass |
