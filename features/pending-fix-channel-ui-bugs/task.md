# Tasks: fix-channel-ui-bugs

## Task Breakdown

### 1. 修复 agentStreams 未清空 bug
- [ ] 修改 `src/lib/useChannel.ts` 中 `channel-response` 事件处理
- [ ] 将 `return prev` 改为 `return allDone ? [] : prev`
- [ ] 验证 single-agent 和 multi-agent 场景

### 2. 修复 MentionAutocomplete agent icon 渲染
- [ ] 在 `src/components/MentionAutocomplete.tsx` 中 import AgentIcon
- [ ] 替换 dropdown item 中的 emoji div 为 AgentIcon 组件
- [ ] 确保选中状态的 bgColor 正确切换

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 分析了两个 bug 的根因 |
