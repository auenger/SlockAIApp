# Tasks: feat-token-usage-ui

## Task Breakdown

### 1. TypeScript 类型定义
- [ ] `types.ts` 添加 `TokenUsage` interface（input_tokens, output_tokens, cache_read_tokens, cache_write_tokens）
- [ ] `StreamEvent` 添加 `token_usage` 可选字段
- [ ] `ChannelMessage` 添加 `token_usage` 可选字段
- [ ] `ThreadMessageData` 添加 `token_usage` 可选字段

### 2. 数据采集 — Channel 模式
- [ ] `useAgentRuntimes.ts` 在 `is_done` 事件处理中提取 `event.token_usage`
- [ ] 将 token_usage 附加到 ChannelMessage 对象
- [ ] 确保持久化到 JSONL 时包含 token_usage

### 3. 数据采集 — Thread 模式
- [ ] `useThreadChat.ts` 在 `is_done` 事件处理中提取 `event.token_usage`
- [ ] 将 token_usage 附加到 ThreadMessageData 对象

### 4. TokenUsageBadge 组件
- [ ] 新建 `src/components/TokenUsageBadge.tsx`
- [ ] 折叠态：显示总 token 数（格式化 1.2k / 1.2M）
- [ ] 展开态（hover/click）：显示按模型的 input/output/cache 分布
- [ ] token_usage 为空时不渲染

### 5. 消息底部集成
- [ ] `MessageContentRenderer.tsx` 消息末尾添加 TokenUsageBadge（agent 消息）
- [ ] `ThreadPanel.tsx` 消息末尾添加 TokenUsageBadge（agent 消息）

### 6. Agent 面板聚合统计
- [ ] `useAgentProfile.ts` 添加 token_usage 聚合计算逻辑（遍历消息历史累加）
- [ ] Agent Profile 页添加 Token 统计卡片组件
- [ ] 按模型分组显示累计用量

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-20 | Feature created | Spec + Tasks defined |
