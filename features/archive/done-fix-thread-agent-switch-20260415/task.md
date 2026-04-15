# Tasks: fix-thread-agent-switch
## Task Breakdown

### 1. App.tsx — 传递 thread props
- [x] 从 useThreadChat 解构出 isStreaming, isThinking, streamingText
- [x] 将 threadActiveThread, threadIsStreaming, threadIsThinking, threadStreamingText, threadSend, threadCreateNewThread 作为 props 传给 MainContent
- [x] 在 handleAgentSelect 中调用 clearActive 确保完全重置

### 2. MainContent.tsx — 移除本地 useThreadChat
- [x] 删除本地 `const { activeThread, isStreaming, isThinking, streamingText, createNewThread, selectThread, send } = useThreadChat()`
- [x] 从 props 接收 thread 状态和操作
- [x] 更新 displayMessages 使用 prop 传入的 threadActiveThread
- [x] 更新 handleSendMessage 使用 prop 传入的 send 和 createNewThread
- [x] 更新 streaming indicator 使用 prop 传入的 isStreaming/isThinking/streamingText

### 3. 类型更新
- [x] 更新 MainContent props 类型定义

### 4. 验证
- [x] 手动测试 Agent 切换对话隔离
- [x] 手动测试新 thread 创建
- [x] 手动测试 sidebar thread 选择

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-15 | Created | Feature request created |
| 2026-04-15 | Implemented | All tasks completed. Removed local useThreadChat from MainContent, unified to single App-level instance. TypeScript + build pass. |
