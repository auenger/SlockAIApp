# Feature: feat-channel-agent-thinking Channel Agent 思考过程渲染

## Basic Information
- **ID**: feat-channel-agent-thinking
- **Name**: Channel Agent 思考过程渲染（Claude Code 输出结构化展示）
- **Priority**: 60
- **Size**: M
- **Dependencies**: None
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-13

## Description

在 Channel 的 Agent Streaming Bubble 中，渲染 Claude Code CLI 的完整结构化输出（tool_use / tool_result），让用户能实时看到 Agent 的思考过程和工具调用。

核心原则：**仅渲染，不保存**。content_blocks 只在流式阶段展示，完成后丢弃，不写入 channel 对话历史。

## User Value Points

### VP1: Tool Call 可视化
用户在 Channel 中 @Agent 后，能实时看到 Agent 正在调用的工具（读文件、写文件、运行命令等），而不是只看到最终文字回复。这提供了：
- 透明度：用户知道 Agent 在做什么
- 调试能力：出问题时能看到卡在哪一步
- 体验提升：等待过程不再空白

### VP2: 流式结构化渲染
将 Claude Code 的 stream-json 输出中的 content_blocks（tool_use、tool_result）以折叠/展开卡片形式展示，清晰区分文字输出和工具调用。

## Context Analysis

### Reference Code
- **AINative `useAgentStream.ts`**: 收集 assistant 类型 chunk 的 text，用 ReactMarkdown 渲染。方案简单但没有 tool_use 渲染。
- **AINative `ProjectView.tsx`**: ReactMarkdown + streaming cursor，没有 content_blocks 处理。

### Related Documents
- Claude Code CLI `--output-format stream-json --verbose` 输出格式：
  - `type: "assistant"` → `message.content` 包含 `[{type: "text", text: "..."}, {type: "tool_use", id: "...", name: "...", input: {...}}]`
  - `type: "assistant"` (后续) → `message.content` 包含 `[{type: "tool_result", tool_use_id: "...", content: "..."}]`
  - `type: "result"` → 最终结果

### Related Features
- `feat-md-rendering` (completed): Markdown 渲染优化 & Tool Call 结构化展示（Thread 模式）
- `fix-channel-msg-render` (completed): Channel 消息即时渲染 & Agent Thinking 状态清理

### 现有数据流
```
Rust StreamEvent.content_blocks → Tauri Event → Frontend ChannelChunkEvent
  ↓
当前：content_blocks 被忽略，只取 text
目标：content_blocks 传入 AgentStreamState，渲染为卡片
```

## Technical Solution

### 1. 扩展 `AgentStreamState` (useChannel.ts)
添加 `content_blocks` 字段：
```typescript
export interface AgentStreamState {
  // ...existing
  contentBlocks: ContentBlock[];  // 新增
}

export interface ContentBlock {
  type: 'tool_use' | 'tool_result';
  id?: string;           // tool_use id
  name?: string;         // tool name (e.g. "Read", "Write", "Bash")
  input?: unknown;       // tool_use input params
  tool_use_id?: string;  // tool_result reference
  content?: string;      // tool_result content
}
```

### 2. 更新 chunk handler (useChannel.ts)
在 `agent://channel-chunk` 监听器中，解析 `streamEvent.content_blocks`，累积到对应 agent 的 stream state。

### 3. 更新 `AgentStreamBubble` (MainContent.tsx)
在现有 MarkdownRenderer 下方，渲染 content_blocks 为折叠卡片：
- **tool_use**: 显示工具名称 + 关键参数（如文件路径、命令），可折叠
- **tool_result**: 显示结果摘要，可折叠
- 样式：轻量卡片，brutal-border 风格，区分于正文

### 4. 不持久化
`content_blocks` 仅存在于流式阶段的 React state，随 `agentStreams` 清理而丢弃。保存到 channel history 时只保存 `text`，不包含 content_blocks。

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我在 Channel 中 @Agent 后，希望能实时看到 Agent 正在调用哪些工具，而不是只看到一个 "Thinking..." 状态。

### Scenarios

#### Scenario 1: tool_use 实时渲染
```gherkin
Given 用户在 Channel 中发送了 @Gaby 帮我读一下 src/main.rs
When Gaby 开始执行并调用 Read 工具
Then Streaming Bubble 中显示工具调用卡片，包含 "Read" 和 "src/main.rs"
And 卡片在流式阶段可见
```

#### Scenario 2: tool_result 渲染
```gherkin
Given Agent 已调用 Read 工具
When 工具返回结果
Then Streaming Bubble 中在 tool_use 卡片下方显示 tool_result 卡片
And 结果内容可折叠查看
```

#### Scenario 3: content_blocks 不持久化
```gherkin
Given Agent 完成回复，流式状态清理
When 用户重新加载 Channel
Then Channel 消息历史中只有文字回复，不包含 tool_use/tool_result 卡片
```

#### Scenario 4: 无 tool 调用时正常工作
```gherkin
Given Agent 回复纯文字（无工具调用）
When 流式完成
Then 显示正常的文字回复，没有工具卡片
And 行为与当前完全一致
```

### UI/Interaction Checkpoints
- tool_use 卡片：工具名 badge + 关键参数预览，点击展开详情
- tool_result 卡片：结果预览（截断），点击展开完整内容
- 折叠/展开有动画过渡
- 卡片在 agent icon 右侧，与正文上下排列

### General Checklist
- 不影响现有 Thread 模式的 tool call 渲染
- content_blocks 为空时无额外 UI 元素
- 性能：大量 tool 调用时不卡顿（虚拟化或限制渲染数量）

## Merge Record

- **Completed**: 2026-04-13T14:30:00+08:00
- **Merged branch**: feature/feat-channel-agent-thinking
- **Merge commit**: 0208bde2c4bd6f8f552de7b6c75a69bf489789c1
- **Archive tag**: feat-channel-agent-thinking-20260413
- **Conflicts**: None during rebase. Stash pop had minor conflict in useChannel.ts (resolved by keeping feature's contentBlocks logic).
- **Verification**: passed (tsc clean, vite build success, 4/4 Gherkin scenarios verified via code analysis)
- **Stats**: started 2026-04-13T14:00:00+08:00, duration ~30min, 1 commit, 3 files changed (+168/-11)
