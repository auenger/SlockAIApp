# Feature: fix-thread-agent-switch Thread 状态隔离 — Agent 切换时对话残留

## Basic Information
- **ID**: fix-thread-agent-switch
- **Name**: Thread 状态隔离 — Agent 切换时对话残留修复
- **Priority**: 95
- **Size**: S
- **Dependencies**: none
- **Parent**: null
- **Children**: none
- **Created**: 2026-04-15

## Description

### Bug 表现
1. 选择 Agent A 对话 → thread 消息正确渲染在 Agent A 的 chat 中
2. 点击 Agent B → 对话消息没有消失，反而 Agent 名称和 icon 被替换为 Agent B 的
3. 无法为 Agent B 创建新 thread，因为旧 thread 的 `activeThread` 状态仍然残留

### 根因分析
存在两个独立的 `useThreadChat()` hook 实例：
- **App.tsx** (line 20): 全局 thread 管理，用于 Sidebar 列表 + ThreadPanel
- **MainContent.tsx** (line 459-467): 本地 thread 管理，用于主聊天区域渲染

当 `handleAgentSelect` 切换 Agent 时：
1. App.tsx 设置 `activeThreadId = null` → 清空了 App 层的状态
2. 但 MainContent 本地的 `activeThread`（来自自己的 `useThreadChat()` 实例）**完全不受影响**
3. MainContent 的 `displayMessages` (line 560-572) 仍从本地 `activeThread` 读取消息
4. 而 `selectedAgent` 已变成 Agent B → sender name/icon 来自 Agent B
5. `handleSendMessage` 中 `if (!threadId)` 条件为 false（本地 activeThread 仍存在）→ 无法创建新 thread

### 修复方案
**统一 useThreadChat 为单一实例**（App.tsx 持有），通过 props 传递给 MainContent：
1. 从 MainContent 移除本地 `useThreadChat()` 调用
2. 在 App.tsx 中将 thread 相关状态（activeThread, isStreaming, isThinking, streamingText, send, createNewThread, selectThread）作为 props 传入 MainContent
3. MainContent 的 `handleSendMessage` 使用 prop 传入的 send/createNewThread
4. 切换 Agent 时通过 App.tsx 统一管理 thread 状态清空

## User Value Points
1. **Agent 切换时对话正确隔离** — 切换 Agent 后不再显示上一个 Agent 的对话
2. **可以正常创建新 thread** — 切换 Agent 后能发起全新对话

## Context Analysis

### Reference Code
- `src/App.tsx` — 全局状态管理，useThreadChat 实例 #1
- `src/components/MainContent.tsx` — 主聊天区域，useThreadChat 实例 #2（问题根源）
- `src/lib/useThreadChat.ts` — Thread hook 定义

### Related Documents
- 无

### Related Features
- `feat-thread-chat` — Thread 对话基础功能
- `fix-channel-state-isolation` — 类似的状态隔离修复（Channel 侧）

## Technical Solution

### Step 1: 扩展 MainContent Props
在 MainContent 组件 props 中新增 thread 相关字段：
```typescript
interface MainContentProps {
  // ...existing props
  // Thread state from App-level useThreadChat
  threadActiveThread: Thread | null;
  threadIsStreaming: boolean;
  threadIsThinking: boolean;
  threadStreamingText: string;
  threadSend: (agentId: string, threadId: string, message: string) => Promise<void>;
  threadCreateNewThread: (agentId: string, agentName: string) => Promise<Thread>;
  threadSelectThread: (agentId: string, threadId: string) => Promise<void>;
}
```

### Step 2: App.tsx 传递 thread props
```tsx
<MainContent
  // ...existing props
  threadActiveThread={activeThread}
  threadIsStreaming={threadIsStreaming}
  threadIsThinking={threadIsThinking}
  threadStreamingText={threadStreamingText}
  threadSend={sendThreadMessage}
  threadCreateNewThread={createNewThread}
  threadSelectThread={selectThread}
/>
```

### Step 3: MainContent 移除本地 useThreadChat
删除 MainContent 内部的 `useThreadChat()` 调用，改用 props。

### Step 4: handleAgentSelect 增强
在 `handleAgentSelect` 中调用 `clearActive()` 确保 thread 状态完全重置：
```typescript
const handleAgentSelect = (agent: AgentWithRuntime) => {
  setSelectedAgent(agent);
  setActiveChannel(null);
  _clearActiveChannel();
  setActiveThreadId(null);
  clearActiveThread(); // 新增：清空 thread 状态
  setIsThreadOpen(false);
  setActiveTab('CHAT');
};
```

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望切换 Agent 时对话能正确隔离，每个 Agent 的对话互不干扰。

### Scenarios

#### Scenario 1: Agent 切换时旧对话消失
```gherkin
Given 用户在 Agent A 的 chat 中有对话记录
When 用户点击 Agent B
Then Agent A 的对话消息消失
And 聊天区域显示空白（等待新对话）
And Agent B 的名称和 icon 正确显示
```

#### Scenario 2: 切换 Agent 后能创建新 thread
```gherkin
Given 用户在 Agent A 的 chat 中有对话记录
When 用户点击 Agent B
And 在输入框中输入消息并发送
Then 为 Agent B 创建新的 thread
And 新消息正确显示在 Agent B 的 chat 中
```

#### Scenario 3: 从 sidebar 选择 thread 正常工作
```gherkin
Given 存在 Agent A 的 thread-1 和 Agent B 的 thread-2
When 用户在 sidebar 点击 thread-2
Then 主聊天区域显示 thread-2 的消息
And Agent B 的名称和 icon 正确显示
```

### UI/Interaction Checkpoints
- Agent 切换时，聊天区域应立即清空（不显示旧消息）
- 输入框应在切换后立即可用
- 思考状态（thinking indicator）不应残留

### General Checklist
- useThreadChat 只有一个实例（App.tsx）
- MainContent 通过 props 接收 thread 状态
- 所有 thread 操作通过 App.tsx 统一管理

## Merge Record
- **Completed**: 2026-04-15T19:30:00+08:00
- **Merged Branch**: feature/fix-thread-agent-switch
- **Merge Commit**: de84f7e
- **Archive Tag**: fix-thread-agent-switch-20260415
- **Conflicts**: None
- **Verification**: All 3 Gherkin scenarios PASS (code analysis)
- **Files Changed**: 2 (src/App.tsx, src/components/MainContent.tsx)
- **Duration**: ~30 min
