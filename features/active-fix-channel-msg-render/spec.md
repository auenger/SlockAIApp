# Feature: fix-channel-msg-render Channel 消息即时渲染 & Agent 状态修复

## Basic Information
- **ID**: fix-channel-msg-render
- **Name**: Channel 消息即时渲染 & Agent Thinking 状态清理
- **Priority**: 85
- **Size**: S
- **Dependencies**: none
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-11T21:00:00+08:00

## Description

修复 Channel 对话中两个关键 UX 问题：
1. 用户发送消息后，消息没有立刻渲染到 Channel 中，需要等待 Agent 响应后用户消息才一起出现
2. Agent 答复完成后，遗留 "Agent THINKING..." 状态没有正确清除

## User Value Points

### VP1: 用户消息即时可见
用户发送消息后应立即看到自己的消息出现在 Channel 中，不应等待后端处理或 Agent 响应。

### VP2: Agent 状态准确切换
Agent 完成响应后，THINKING 状态应立即清除，不应残留。多 Agent 场景中每个 Agent 的状态都应独立且正确地管理。

## Context Analysis

### Reference Code
- `src/lib/useChannel.ts` — Channel 状态管理 hook，包含 send() 函数和事件监听
- `src/components/MainContent.tsx` — Channel 消息渲染和发送入口
- `src/components/AgentStreamBubble.tsx` — Agent 流式响应气泡组件

### Root Cause Analysis

**Bug 1 — 用户消息延迟渲染：**
- `useChannel.ts:384` 中 `const updatedChannel = await sendChannelMessage(channelId, message)` 是异步 IPC 调用
- `setActiveChannel(updatedChannel)` 要等后端返回才执行
- 后端处理包含：解析 @mention、resolve agents、保存 JSONL + SQLite
- 在此期间用户看不到自己的消息

**Bug 2 — THINKING 状态残留：**
- `useChannel.ts:536-561` 的 `channel-response` 事件处理器中，`allDone` 依赖 `prev.every(s => s.done)`
- `done` 标记由 `channel-chunk` 事件设置（line 509），但 `channel-response` 和 `channel-chunk(is_done)` 是两个独立事件
- 竞态条件：如果 `channel-response` 先到达而 `channel-chunk(is_done)` 还没处理，`allDone` 为 false
- 结果：流状态不会被清理，THINKING 持续显示
- 30s fallback timeout (line 567) 只检查 `prev.length === 0`，无法处理有残留流的情况

### Related Features
- `fix-channel-ui-bugs` — 之前修复的 thinking 状态和 icon 渲染问题
- `fix-delete-and-render` — 之前的删除和渲染状态修复
- `feat-channel-zone-protocol` — Channel Zone Protocol 实现

## Technical Solution

### Fix 1: Optimistic Update 用户消息

在 `useChannel.ts` 的 `send()` 函数中，在调用 `sendChannelMessage` IPC 之前，乐观地将用户消息插入 `activeChannel.messages`：

```typescript
// 在 send() 函数中，isTauri 分支，sendChannelMessage 调用之前：
const optimisticUserMsg: ChannelMessage = {
  id: `msg-pending-${Date.now()}`,
  channel_id: channelId,
  sender_type: "user",
  sender_id: "user",
  content: message,
  timestamp: new Date().toISOString(),
};
setActiveChannel((prev) => {
  if (!prev || prev.id !== channelId) return prev;
  return { ...prev, messages: [...prev.messages, optimisticUserMsg] };
});

// 然后 IPC 返回后用真实数据替换
const updatedChannel = await sendChannelMessage(channelId, message);
setActiveChannel(updatedChannel); // 用后端真实数据覆盖（包含正确的 message ID）
```

### Fix 2: Agent Thinking 状态清理

在 `channel-response` 事件中，不依赖 `channel-chunk` 的 `done` 标记，而是直接在 response 事件中标记该 agent 为 done：

```typescript
// channel-response handler 中，先标记该 agent 为 done
setAgentStreams((prev) =>
  prev.map((s) =>
    s.agent_id === agent_id
      ? { ...s, streaming: false, thinking: false, done: true }
      : s
  )
);

// 然后再检查是否全部完成
setAgentStreams((prev) => {
  const allDone = prev.every((s) => s.done);
  if (allDone) {
    setStreamingText("");
    setIsStreaming(false);
    setIsThinking(false);
    // cleanup listeners...
  }
  return allDone ? [] : prev;
});
```

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望发送消息后立即看到自己的消息，并在 Agent 完成后看到正确的状态变化。

### Scenarios

```gherkin
Scenario: 用户发送消息后立即看到消息
  Given 用户在 Channel 中输入了消息 "@Agent 你好"
  When 用户按下发送按钮
  Then 用户消息应在 <100ms 内出现在 Channel 消息列表中
  And 不需要等待 Agent 响应

Scenario: 单 Agent 完成响应后 THINKING 状态清除
  Given Channel 中有一个 Agent 正在 THINKING
  When Agent 完成响应并发送 channel-response 事件
  Then THINKING 状态应立即清除
  And Agent 回复消息应显示在消息列表中
  And 不应残留任何 "THINKING..." 文字

Scenario: 多 Agent 依次完成后状态正确清除
  Given Channel 中有 3 个 Agent 按顺序响应
  When 第 1 个 Agent 完成响应
  Then 第 1 个 Agent 显示为 Done
  And 其他 Agent 保持各自状态不变
  When 所有 Agent 完成响应
  Then 所有 THINKING/Streaming 状态清除
  And AgentStreamBubble 消失
  And 输入框恢复可用

Scenario: Agent 响应出错时状态正确清除
  Given Channel 中 Agent 正在 THINKING
  When Agent 运行时发生错误（如 runtime 不可用）
  Then THINKING 和 Streaming 状态都应清除
  And 输入框恢复可用
  And 错误信息显示给用户
```

### General Checklist
- [ ] 不引入新的状态管理问题
- [ ] 多 Agent 场景下每个 Agent 状态独立正确
- [ ] A2A 触发场景下状态仍然正确
- [ ] 乐观更新在 IPC 失败时有合理的错误处理
