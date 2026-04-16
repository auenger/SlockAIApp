# Feature: fix-thread-streaming-ux Thread/Agent Chat Thinking & Streaming 效果对齐 Channel

## Basic Information
- **ID**: fix-thread-streaming-ux
- **Name**: Thread/Agent Chat Thinking & Streaming 效果对齐 Channel
- **Priority**: 75
- **Size**: M
- **Dependencies**: none
- **Parent**: null
- **Children**: empty
- **Created**: 2026-04-16T22:30:00+08:00

## Description

Thread 面板和 MainContent Thread 模式的 thinking/streaming 效果与 Channel 的 AgentStreamBubble 存在显著差异，需要对齐以保持 UX 一致性。

### 当前差异分析

| 维度 | Channel (参考实现) | Thread/Agent (当前) |
|------|-------------------|-------------------|
| **Thinking 动画** | "Thinking" 标签 + 灰色跳动圆点 + statusMessage | `animate-pulse` 容器 + 灰色占位条 |
| **Streaming 状态** | "Streaming..." + 文字 + cyan 跳动圆点 + ContentBlock 卡片 | "Streaming..." + 文字 + 圆点 (无 ContentBlock) |
| **Done 状态** | 绿色 "Done" 标签 | 无 |
| **Status 消息** | 显示 runtime 状态 (如 "Session initialized · claude-sonnet-4") | 不显示 |
| **Content Blocks** | 显示 tool_use/tool_result 卡片 | 不显示 |
| **Agent Start 事件** | 后端发送 `agent://channel-agent-start` | 无对应事件 |

### 涉及文件

**前端 (主要修改)**:
- `src/lib/useThreadChat.ts` — 增加 contentBlocks 和 statusMessage 追踪
- `src/components/ThreadPanel.tsx` — 对齐 thinking/streaming UI
- `src/components/MainContent.tsx` — Thread 模式的 thinking/streaming 对齐

**后端 (次要修改)**:
- `src-tauri/src/commands/thread.rs` — 发送 `agent://thread-agent-start` 事件

## User Value Points

1. **Thinking 动画一致性** — Thread 中 Agent 思考时显示与 Channel 相同的跳动圆点 + 状态消息，而非简单的灰色占位条
2. **Streaming 过程可视化** — Thread 中展示 tool call 过程 (ContentBlock 卡片) 和 runtime 状态消息，让用户了解 Agent 正在做什么
3. **完成状态指示** — Agent 完成后显示 "Done" 状态标签

## Context Analysis

### Reference Code
- `src/components/MainContent.tsx:216-300` — AgentStreamBubble (Channel 参考实现)
- `src/lib/useChannel.ts:37-68` — AgentStreamState / ChannelStreamState 类型定义
- `src/components/ThreadPanel.tsx:216-259` — Thread 当前的 thinking/streaming 实现
- `src/components/MainContent.tsx:1151-1194` — MainContent Thread 模式的 thinking/streaming

### Related Documents
- `src/types.ts` — ContentBlock, StreamEvent 类型定义

### Related Features
- feat-thread-ux-polish (已完成) — Thread 面板 Thinking/Streaming 动画 + 宽度调整优化
- feat-channel-agent-thinking (已完成) — Channel Agent 思考过程渲染
- fix-channel-state-isolation (已完成) — Channel 切换时 Agent Thinking/Streaming 状态隔离

## Technical Solution

### 1. useThreadChat Hook 增强

在 `useThreadChat.ts` 中增加：
- `contentBlocks: ContentBlock[]` — 收集 streaming 过程中的 tool_use/tool_result 块
- `statusMessage?: string` — 追踪 runtime 状态消息 (来自 system 类型事件)
- 监听 `agent://chunk` 事件时处理 `content_blocks` 和 system 事件
- 在 `send()` 中重置新增状态

### 2. 后端 Thread Agent Start 事件

在 `thread.rs send_message` 中，在 runtime 执行前发送 `agent://thread-agent-start` 事件：
```json
{
  "thread_id": "...",
  "agent_id": "...",
  "runtime_id": "claude-code",
  "runtime_name": "Claude Code"
}
```

### 3. ThreadPanel.tsx UI 对齐

- **Thinking**: 替换 `animate-pulse` + 灰色占位条为 Channel 风格的 "Thinking" + 跳动灰色圆点 + statusMessage
- **Streaming**: 添加 ContentBlock 卡片渲染 + cyan 跳动圆点
- **Done**: 添加 "Done" 绿色标签

### 4. MainContent.tsx Thread 模式 UI 对齐

与 ThreadPanel 相同的 UI 调整，确保两处 Thread 展示效果一致。

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望在 Thread 和 Agent Chat 中看到与 Channel 一致的 thinking/streaming 效果，这样无论在哪种模式下都能清晰了解 Agent 的执行状态。

### Scenarios (Given/When/Then)

#### Scenario 1: Thread Thinking 动画效果
```gherkin
Given 用户在 Thread 面板中向 Agent 发送消息
When Agent 开始思考但尚未返回文字
Then 应显示 Agent 名称 + "Thinking" 标签 + 3个灰色跳动圆点
And 如果有 statusMessage 则显示在 Thinking 标签下方
And 不应显示 animate-pulse 灰色占位条
```

#### Scenario 2: Thread Streaming 展示 Tool Call 过程
```gherkin
Given Agent 正在 Thread 中流式响应
When 收到包含 content_blocks 的 chunk 事件
Then 应在文字下方显示 ContentBlock 卡片 (tool_use/tool_result)
And 应在文字末尾显示 3个 cyan 跳动圆点
And 应显示 "Streaming..." 状态标签
```

#### Scenario 3: Thread Agent 完成状态
```gherkin
Given Agent 在 Thread 中完成响应
When streaming 结束且最后一个 chunk 的 is_done 为 true
Then 应短暂显示 "Done" 绿色状态标签
And ContentBlock 卡片应保留显示
```

#### Scenario 4: MainContent Thread 模式效果一致
```gherkin
Given 用户在 MainContent 的 Thread 模式下
When Agent 正在思考或流式响应
Then thinking 和 streaming 效果应与 ThreadPanel 中的效果一致
And 应与 Channel 的 AgentStreamBubble 风格对齐
```

### UI/Interaction Checkpoints
- Thinking 状态：3个灰色圆点跳动动画 (staggered: 0ms, 150ms, 300ms)
- Streaming 状态：3个 cyan 圆点跳动动画 + 文字实时更新
- Done 状态：绿色 "Done" 标签
- ContentBlock 卡片：与 Channel 中的 ContentBlockCard 组件复用
- Status 消息：灰色等宽字体斜体显示

### General Checklist
- [ ] 不影响 Channel 现有的 thinking/streaming 效果
- [ ] Thread 和 Agent Chat 的效果与 Channel 视觉一致
- [ ] ContentBlock 卡片与 Channel 复用同一组件
- [ ] streaming 结束后 contentBlocks 正确清理
