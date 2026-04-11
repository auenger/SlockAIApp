# Tasks: fix-channel-msg-render

## Task Breakdown

### 1. useChannel.ts — 乐观更新用户消息
- [ ] 在 `send()` 函数的 isTauri 分支中，`sendChannelMessage` 调用前插入乐观用户消息
- [ ] IPC 返回后用后端真实数据覆盖乐观消息

### 2. useChannel.ts — Agent 状态清理修复
- [ ] 在 `channel-response` 事件处理器中，先将对应 agent 标记为 done
- [ ] 确保全部完成后清理 isStreaming/isThinking/agentStreams
- [ ] 验证 chunk 事件中的 is_done 处理仍正确（双重保障）

### 3. 验证
- [ ] 单 Agent 场景：发送消息 → 立即看到用户消息 → Agent THINKING → Agent 回复 → THINKING 消失
- [ ] 多 Agent 场景：依次完成，状态各自正确
- [ ] 错误场景：runtime 不可用时状态正确清除

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-11 | Feature created | 等待开发 |
