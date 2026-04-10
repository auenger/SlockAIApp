# Feature: feat-thread-panel-live ThreadPanel 真实功能

## Basic Information
- **ID**: feat-thread-panel-live
- **Name**: ThreadPanel 真实数据接入
- **Priority**: 60
- **Size**: S
- **Dependencies**: feat-thread-chat (completed)
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-09

## Description

当前 `ThreadPanel.tsx` 是完全硬编码的静态 Mock UI（写死的代码块、表格、消息内容）。需要将其替换为真实功能：接收 thread 数据 props，展示真实的 Thread 消息列表，支持消息输入和发送。可作为 Channel 详情面板或 Thread 详情侧边栏使用。

## User Value Points

1. **Thread 详情展示** — 用户点击右侧面板可查看当前 Thread 的完整对话记录和详情信息

## Context Analysis

### Reference Code
- `src/components/ThreadPanel.tsx` — 当前全静态 Mock（需完全重写）
- `src/components/MainContent.tsx` — 已有 Thread 消息渲染逻辑可参考
- `src/lib/useThreadChat.ts` — Thread chat hook 已完整实现
- `src/types.ts` — Thread, ThreadMessageData 类型

### Related Features
- feat-thread-chat (completed) — Thread 1:1 对话

## Technical Solution

1. ThreadPanel 接收 props: `thread: Thread | null`, `agent: AgentWithRuntime | null`, `onSend`, `onClose`
2. 展示 thread 消息列表（复用 MainContent 中的消息渲染模式）
3. 底部输入框支持发送消息（调用 `onSend`）
4. Header 显示 agent 名称和 thread 标题
5. 无 thread 选中时显示空状态提示

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我想在右侧面板查看当前 Thread 的详细对话记录，方便上下文参考。

### Scenarios (Given/When/Then)

#### Scenario 1: 展示 Thread 消息
```gherkin
Given 用户已选中一个 Agent 和 Thread
When ThreadPanel 打开
Then 显示该 Thread 的所有消息
And 消息区分用户消息和 Agent 消息
And 消息按时间顺序排列
```

#### Scenario 2: 发送消息
```gherkin
Given ThreadPanel 展示了一个活跃 Thread
When 用户在输入框输入消息并点击 Send
Then 消息发送到该 Thread
And 消息列表实时更新
```

#### Scenario 3: 空状态
```gherkin
Given 用户未选中任何 Thread
When ThreadPanel 打开
Then 显示 "Select a thread to view details" 提示
```

### UI/Interaction Checkpoints
- 遵循 brutal-border 风格
- 消息滚动到底部
- 关闭按钮（X）正常工作

### General Checklist
- [ ] 移除所有硬编码内容
- [ ] TypeScript props 类型正确
- [ ] 无遗留 Mock 数据
