# Feature: feat-thread-ux-polish Thread 面板 UX 打磨

## Basic Information
- **ID**: feat-thread-ux-polish
- **Name**: Thread 面板 UX 打磨（Thinking/Streaming 动画 + 宽度调整优化）
- **Priority**: 75
- **Size**: M
- **Dependencies**: feat-thread-chat (已完成)
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-15

## Description
Thread 面板目前缺少关键的 UX 反馈机制：用户发送消息后，在等待 AI 回复期间没有任何视觉反馈（无 thinking 指示、无 streaming 动画），消息会突然跳出来，体验突兀。同时 Thread 面板的宽度调整功能需要优化，提升可发现性。

## User Value Points

### VP1: Thinking/Streaming 动画指示
用户发送消息后，能看到 AI 正在思考和流式输出的视觉反馈，而不是干等后突然出现消息。

### VP2: 面板宽度调整优化
Thread 面板宽度可调，用户可以拖拽调整到舒适的宽度，且调整操作易于发现。

## Context Analysis

### Reference Code
- `src/components/ThreadPanel.tsx` — Thread 面板主组件，当前无 streaming 动画
- `src/lib/useThreadChat.ts` — Thread chat hook，已有 `isStreaming`、`isThinking`、`streamingText` 状态
- `src/components/MainContent.tsx` (L1175-1197) — MainContent 的 streaming 动画实现（三点弹跳）
- `src/lib/useResizable.ts` — 已有的 resize hook
- `src/App.tsx` — 面板布局，ThreadPanel 初始宽度 320px

### Related Documents
- 已完成的 `feat-channel-agent-thinking` — Channel 的 thinking 渲染实现
- 已完成的 `fix-channel-state-isolation` — Channel 状态隔离

### Related Features
- `feat-thread-chat` (已完成) — Thread 对话基础功能

## Technical Solution

### Props Extension
Added `isThinking`, `isStreaming`, `streamingText` optional props to ThreadPanel. App.tsx passes these from the existing useThreadChat hook (already had these states, just not wired to ThreadPanel).

### Thinking Indicator
- `animate-pulse` container with AgentIcon + "Thinking..." label + gray progress bar
- Condition: `isThinking && !streamingText` (transitions away when first chunk arrives)

### Streaming Indicator
- MarkdownRenderer for real-time text + 3-dot bounce animation (`animate-bounce`, staggered 0/150/300ms)
- Color: `bg-brutal-cyan` matching MainContent pattern
- Condition: `isStreaming && streamingText`

### Resize Handle Enhancement
- Widened to `w-1.5`, added `group` class with grip dots (3x 4px circles) appearing on hover
- Range adjusted from 240-560 to 280-600

### Auto-scroll
- Extended useEffect dependency from `[thread?.messages]` to `[thread?.messages, streamingText]`

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望在 Thread 中与 Agent 对话时，能看到 AI 的思考和回复过程，而不是干等后突然出现消息，这样我能知道系统正在工作。

### Scenarios

#### Scenario 1: Thinking 状态指示
```gherkin
Given 用户在 Thread 面板中与 Agent 对话
When 用户发送一条消息，Agent 开始处理
Then 应显示 "Thinking..." 或类似的思考中动画指示器
And 动画应包含视觉反馈（如弹跳点、脉冲动画）
And 当 Agent 开始输出文本时，思考指示应过渡到 streaming 状态
```

#### Scenario 2: Streaming 流式输出动画
```gherkin
Given Agent 正在流式输出回复
Then 应在消息区域实时显示流式文本内容
And 文本末尾应显示 streaming 动画指示（如三点弹跳）
And streaming 动画应与 MainContent 中的实现风格一致
When 流式输出完成时
Then 动画指示器消失，显示完整消息
```

#### Scenario 3: 面板宽度调整
```gherkin
Given Thread 面板已打开
When 用户拖拽面板左边缘
Then 面板宽度应平滑跟随拖拽
And 宽度应在合理范围内（如 280px - 600px）
And 拖拽手柄应有视觉提示（hover 高亮等）
```

#### Scenario 4: 无回复时的空状态
```gherkin
Given Thread 面板刚打开或对话为空
When 没有 Agent 回复正在进行
Then 不应显示任何 thinking/streaming 动画
And 面板应保持干净的状态
```

### UI/Interaction Checkpoints
- [x] Thinking 指示器：弹跳动画或脉冲动画，位于消息区域
- [x] Streaming 文本：实时显示，末尾有三点弹跳动画
- [x] 宽度拖拽手柄：左边缘，hover 有视觉反馈
- [x] 动画风格与 MainContent 中 brutalist 主题一致

### General Checklist
- [x] 动画不应影响面板性能
- [x] streaming 状态应在消息完成后正确清理
- [x] 宽度调整不应导致内容溢出或布局错乱

## Merge Record

- **Completed**: 2026-04-16T20:30:00+08:00
- **Merged Branch**: feature/feat-thread-ux-polish
- **Merge Commit**: 1b90022
- **Archive Tag**: feat-thread-ux-polish-20260416
- **Conflicts**: none
- **Verification**: passed (4/4 Gherkin scenarios)
- **Stats**: 1 commit, 2 files changed, 87 insertions, 9 deletions
