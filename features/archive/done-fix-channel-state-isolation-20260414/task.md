# Tasks: fix-channel-state-isolation

## Task Breakdown

### 1. 重构 useChannel.ts 状态为 Per-Channel Map
- [x] 定义 `ChannelStreamState` interface（isStreaming, isThinking, streamingText, agentStreams）
- [x] 将 4 个独立 useState 替换为 `Map<channelId, ChannelStreamState>`
- [x] 封装 helper 函数：`getStreamState(channelId)` / `setStreamState(channelId, partial)`
- [x] 更新 hook 返回值：从 activeChannel 对应的 Map entry 派生 isStreaming/isThinking 等
- [x] 确保所有 `setIsStreaming`/`setIsThinking`/`setStreamingText`/`setAgentStreams` 调用点改为 Map 操作

### 2. 修复 selectChannel 切换逻辑
- [x] selectChannel 切换时不丢弃原 channel 的 streaming 状态（保留在 Map 中）
- [x] 加载目标 channel 时从 Map 恢复其 streaming 状态
- [x] App.tsx 中 handleChannelSelect 适配（如需要）

### 3. 修复 Event Listeners
- [x] 确认所有 Tauri event listeners 的 state 更新写入正确 channelId 的 Map entry
- [x] session-complete 事件清理对应 channel 的 Map entry

### 4. 消费端适配
- [x] 检查 MainContent.tsx 对 isStreaming/isThinking 的使用
- [x] 检查其他消费这些状态的组件

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-14 | Feature created | Bug report + root cause analysis |
| 2026-04-14 | All tasks completed | Per-channel Map refactored, no consumer changes needed |
