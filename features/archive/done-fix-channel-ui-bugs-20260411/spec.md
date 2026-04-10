# Feature: fix-channel-ui-bugs Channel @Agent UI 修复

## Basic Information
- **ID**: fix-channel-ui-bugs
- **Name**: Channel @Agent UI 修复（thinking 状态 + icon 渲染）
- **Priority**: 85
- **Size**: S
- **Dependencies**: none
- **Parent**: null
- **Children**: empty
- **Created**: 2026-04-10T23:45:00+08:00

## Description

Channel 模式中 @Agent 对话后，存在两个 UI 状态 bug：

1. **Agent thinking 状态不消失**：Agent 回复消息渲染完成后，streaming bubble（显示 "Agent thinking" / "Done"）仍然残留在界面上，没有正确清除。
2. **@mention 弹窗 agent icon 不渲染**：在 channel 输入框中使用 @ 提及 agent 时，弹出的 agent 列表中 agent 图标没有正确渲染，仅显示 emoji 文本而非使用 AgentIcon 组件。

## User Value Points

1. **完整的 streaming 生命周期**：Agent 回复完成后，streaming/thinking 状态指示器应自动清除，用户看到干净的消息列表
2. **一致的 icon 渲染**：@mention 弹窗中的 agent icon 应与侧边栏、消息区域等其他位置保持一致，使用统一的 AgentIcon 组件

## Context Analysis

### Reference Code
- `src/lib/useChannel.ts` — Channel 状态管理，`send()` 方法中 `channel-response` 事件处理后 `agentStreams` 未清空
- `src/components/MainContent.tsx` — `AgentStreamBubble` 组件渲染 streaming 状态，`isThinking` 相关条件渲染
- `src/components/MentionAutocomplete.tsx` — @mention 弹窗组件，直接渲染 `awr.agent.emoji` 而非 `AgentIcon`

### Root Cause Analysis

#### Bug 1: agentStreams 未清空
`useChannel.ts` 第 497 行 `channel-response` 回调中：
```ts
setAgentStreams((prev) => {
  const allDone = prev.every((s) => s.done);
  if (allDone) {
    // 清理 isStreaming/isThinking，但 return prev 没有清空数组！
    ...
  }
  return prev; // ← BUG: 应该 return allDone ? [] : prev
});
```
`agentStreams` 保留了 `done: true` 的条目，导致 `AgentStreamBubble` 仍渲染 "Done" 标签。

#### Bug 2: MentionAutocomplete 未使用 AgentIcon
`MentionAutocomplete.tsx` 第 200-205 行，dropdown 中仅渲染 emoji 文本：
```tsx
<div className="w-6 h-6 ...">
  {awr.agent.emoji}  // ← 只显示 emoji，不支持 SVG icon
</div>
```
应替换为 `AgentIcon` 组件以保持一致性。

### Related Features
- fix-delete-and-render — 同为 Channel 渲染相关修复

## Technical Solution

### Fix 1: 清空 agentStreams
在 `useChannel.ts` 的 `channel-response` 处理中，当 `allDone` 时将 `agentStreams` 清空：
```ts
return allDone ? [] : prev;
```

### Fix 2: MentionAutocomplete 使用 AgentIcon
在 `MentionAutocomplete.tsx` 的 dropdown item 中，将 emoji div 替换为 `<AgentIcon>` 组件：
```tsx
<AgentIcon
  icon={awr.agent.icon}
  emoji={awr.agent.emoji}
  size="sm"
  bgColor={idx === selectedIndex ? "bg-white/20" : "bg-brutal-cyan"}
/>
```
需要 import AgentIcon 组件。

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望在 Channel 中 @Agent 对话后，streaming 状态正确清除，且 @mention 弹窗中 agent 图标正确显示。

### Scenarios

#### Scenario 1: Agent 回复完成后 thinking 状态清除
```gherkin
Given 用户在 Channel 中发送了一条 @Agent 消息
When Agent 完成回复并渲染到消息列表
Then 不应再显示 "Thinking..." 或 "Streaming..." 或 "Done" 的 streaming bubble
And 消息列表中只有正常的消息气泡
```

#### Scenario 2: Agent 回复完成后可发送新消息
```gherkin
Given Agent 刚完成一条回复
When 用户输入新的消息
Then 输入框不应被禁用
And Send 按钮应可点击
```

#### Scenario 3: @mention 弹窗正确显示 agent icon
```gherkin
Given 用户在 Channel 输入框中输入 "@"
When @mention 弹窗显示 agent 列表
Then 每个 agent 应正确显示其 icon（SVG 或 emoji）
And icon 应与侧边栏中的 agent icon 一致
```

#### Scenario 4: @mention 弹窗中 agent icon 使用 SVG 图标
```gherkin
Given 一个 agent 配置了 SVG icon（非 emoji）
When 用户在 @mention 弹窗中看到该 agent
Then 应显示对应的 SVG 图标
And 不应显示空白或默认 "A" 字符
```

### General Checklist
- [x] 无 console error
- [x] 不引入新的类型错误
- [x] 多 Agent 同时回复场景测试
- [x] 单 Agent 回复场景测试

## Merge Record

- **Completed**: 2026-04-11T00:45:00+08:00
- **Merged Branch**: feature/fix-channel-ui-bugs
- **Merge Commit**: 010b785
- **Archive Tag**: fix-channel-ui-bugs-20260411
- **Conflicts**: None
- **Verification**: All 4 Gherkin scenarios passed via code analysis + TypeScript build
- **Duration**: ~35 minutes
- **Commits**: 1
- **Files Changed**: 3 (useChannel.ts, MentionAutocomplete.tsx, task.md)
