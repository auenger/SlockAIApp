# Tasks: feat-channel-agent-thinking

## Task Breakdown

### 1. 类型定义
- [x] 在 `types.ts` 中定义 `ContentBlock` 接口（tool_use / tool_result）
- [x] 更新 `StreamEvent` 的 `content_blocks` 类型为 `ContentBlock[]`

### 2. AgentStreamState 扩展 (useChannel.ts)
- [x] 在 `AgentStreamState` 中添加 `contentBlocks: ContentBlock[]`
- [x] 在 chunk handler 中解析 `streamEvent.content_blocks`，累积到 stream state
- [x] 确保 agent-start 时清空 contentBlocks

### 3. Tool Call 卡片组件 (MainContent.tsx)
- [x] 创建 `ContentBlockCard` 子组件：渲染单个 tool_use / tool_result
- [x] tool_use：显示工具名 badge + 参数预览，可折叠
- [x] tool_result：显示结果摘要，可折叠
- [x] 在 `AgentStreamBubble` 中集成，位于 MarkdownRenderer 下方

### 4. 样式与交互
- [x] brutal-border 风格卡片
- [x] 折叠/展开动画
- [x] 流式完成后 contentBlocks 自动清理

### 5. 验证
- [x] TypeScript 编译通过 (tsc --noEmit)
- [x] Vite build 成功
- [ ] 发送带 @mention 的 channel 消息，验证 tool_use 渲染（需手动测试）
- [ ] 验证完成后 content_blocks 不出现在 channel history 中（需手动测试）
- [ ] 验证纯文字回复无额外 UI 元素（需手动测试）

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-13 | Feature created | Spec + tasks defined |
| 2026-04-13 | Implementation complete | types.ts, useChannel.ts, MainContent.tsx all updated; tsc + vite build pass |
