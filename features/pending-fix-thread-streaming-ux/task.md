# Tasks: fix-thread-streaming-ux

## Task Breakdown

### 1. Backend — Thread Agent Start 事件
- [ ] 在 `thread.rs send_message` 中 runtime 执行前 emit `agent://thread-agent-start` 事件
- [ ] 包含 thread_id, agent_id, runtime_id, runtime_name

### 2. useThreadChat Hook 增强
- [ ] 新增 `contentBlocks: ContentBlock[]` 状态
- [ ] 新增 `statusMessage?: string` 状态
- [ ] 监听 `agent://thread-agent-start` 事件设置 statusMessage
- [ ] 处理 `agent://chunk` 事件中的 content_blocks 和 system 事件
- [ ] `send()` 中重置新增状态
- [ ] `clearActive()` 中清理新增状态
- [ ] 在 hook 返回值中导出新增状态

### 3. ThreadPanel.tsx UI 对齐
- [ ] 替换 Thinking 指示器：去掉 `animate-pulse` + 灰色占位条，改为 "Thinking" + 跳动灰色圆点 + statusMessage
- [ ] 增强 Streaming 指示器：添加 ContentBlock 卡片渲染
- [ ] 添加 Done 状态显示

### 4. MainContent.tsx Thread 模式 UI 对齐
- [ ] 替换 Thread 模式 Thinking 指示器为 Channel 风格
- [ ] 增强 Thread 模式 Streaming 指示器，添加 ContentBlock 卡片
- [ ] 添加 Done 状态显示
- [ ] 从 useThreadChat 接收新增的 contentBlocks 和 statusMessage

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-16 | Feature created | 初始任务拆解 |
