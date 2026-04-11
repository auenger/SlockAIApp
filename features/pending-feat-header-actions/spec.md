# Feature: feat-header-actions Header 操作按钮逻辑实现

## Basic Information
- **ID**: feat-header-actions
- **Name**: Header 操作按钮（暂停/刷新/删除）逻辑实现
- **Priority**: 80
- **Size**: S
- **Dependencies**: none
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-11T20:00:00+08:00

## Description

MainContent Header 区域右侧有三个操作按钮（暂停/停止、刷新、删除），目前均为空壳，需要接入真实逻辑。

### 现状
- **暂停按钮** (Square icon): 无 onClick handler，无任何功能
- **刷新按钮** (RotateCcw icon): 无 onClick handler，无任何功能
- **删除按钮** (Trash2 icon): 无 onClick handler，Sidebar 中已有完整删除逻辑（含确认弹窗），需复用

### 需求
1. **删除按钮**: 兼容当前 Sidebar 的删除逻辑，根据当前视图上下文删除 Channel 或 Agent（带确认弹窗）
2. **刷新按钮**: 刷新当前活跃视图的数据（Channel 消息、Thread 消息等）
3. **暂停按钮**: 停止当前正在运行的 Agent Runtime Session

## User Value Points

1. **删除操作统一入口**: 用户可在 Header 直接删除当前 Channel/Agent，无需回到 Sidebar
2. **刷新当前视图**: 用户可一键刷新当前 Channel 或 Thread 的数据，获取最新状态
3. **暂停 Agent 执行**: 用户可随时停止正在运行的 Agent 对话，控制资源消耗

## Context Analysis

### Reference Code
- Header 按钮位置: `src/components/MainContent.tsx:481-491`
- Sidebar 删除逻辑: `src/components/Sidebar.tsx:110-117` (confirmDelete)
- Sidebar 确认弹窗: `src/components/Sidebar.tsx:476-486`
- Channel 删除 IPC: `src/lib/ipc.ts:263` (delete_channel)
- Thread 删除 IPC: `src/lib/ipc.ts:198` (delete_thread)
- Agent 删除 IPC: `src/lib/ipc.ts:119` (delete_agent)
- Runtime 停止: `src/lib/useAgentRuntimes.ts:108-114` (stopSession -> runtime_session_stop)
- Channel 数据加载: `src/lib/useChannel.ts:134` (loadChannels)
- Channel 选择/加载: `src/lib/useChannel.ts:202` (selectChannel)
- App.tsx 删除 handlers: `src/App.tsx:128-163` (handleDeleteChannel/Thread/Agent)

### Related Documents
- Sidebar 已实现完整的删除交互模式（确认弹窗 -> 执行删除 -> 清理状态）

### Related Features
- `fix-delete-and-render` (已完成): 修复了删除功能与渲染状态逻辑

## Technical Solution

### 实现方案

#### 1. 删除按钮
- MainContent 接收 `onDeleteChannel` 和 `onDeleteAgent` 回调 props（复用 App.tsx 中已有逻辑）
- 点击删除按钮时，根据 `isChannelMode` 决定删除 Channel 还是当前选中 Agent
- 在 MainContent 内部实现确认弹窗（参考 Sidebar 的 confirmDelete 模式）
- 删除成功后清理当前视图状态

#### 2. 刷新按钮
- MainContent 接收 `onRefresh` 回调 prop
- Channel 模式: 重新调用 `selectChannel(channelId)` 重新加载 Channel 数据和消息
- Agent/Thread 模式: 重新加载 Thread 消息历史
- 刷新时显示 loading 状态（RotateCcw 图标旋转动画）

#### 3. 暂停按钮
- MainContent 接收 `onStopSession` 回调 prop 和 `channelIsStreaming` 状态
- Channel 模式: 停止当前 Channel 中正在执行的 Agent 对话
- Agent/Thread 模式: 调用 `runtime_session_stop` 停止会话
- 仅在有活跃 Session 时可点击（否则 disabled 样式）

### Props 变更

```typescript
interface MainContentProps {
  // ... 现有 props
  onDeleteChannel?: (channelId: string) => void;
  onDeleteAgent?: (agentId: string) => void;
  onRefresh?: () => void;
  onStopSession?: () => void;
}
```

## Acceptance Criteria (Gherkin)

### User Story
作为一个用户，我希望在 Header 区域通过操作按钮直接管理当前 Channel/Agent，以便快速执行删除、刷新和暂停操作。

### Scenarios (Given/When/Then)

#### Scenario 1: 删除当前 Channel
```gherkin
Given 用户选中了一个 Channel
When 用户点击 Header 的删除按钮
Then 弹出确认弹窗
And 用户确认后删除该 Channel
And 视图回到空白状态
```

#### Scenario 2: 删除当前 Agent
```gherkin
Given 用户选中了一个 Agent（非 Channel 模式）
When 用户点击 Header 的删除按钮
Then 弹出确认弹窗
And 用户确认后删除该 Agent
And 视图回到空白状态
```

#### Scenario 3: 未选中任何内容时删除按钮不可用
```gherkin
Given 用户未选中任何 Channel 或 Agent
When 用户查看 Header 删除按钮
Then 删除按钮呈 disabled 状态
```

#### Scenario 4: 刷新当前 Channel
```gherkin
Given 用户选中了一个 Channel 并查看聊天消息
When 用户点击 Header 的刷新按钮
Then Channel 数据重新加载
And 按钮显示旋转动画表示正在刷新
```

#### Scenario 5: 刷新当前 Thread
```gherkin
Given 用户选中了一个 Agent 并进入 Thread 对话
When 用户点击 Header 的刷新按钮
Then Thread 消息历史重新加载
```

#### Scenario 6: 暂停正在执行的 Agent
```gherkin
Given Channel 中有 Agent 正在生成回复（streaming 状态）
When 用户点击 Header 的暂停按钮
Then Agent 执行被终止
And streaming 状态结束
```

#### Scenario 7: 无活跃会话时暂停按钮不可用
```gherkin
Given 当前没有 Agent 正在执行
When 用户查看 Header 暂停按钮
Then 暂停按钮呈 disabled 状态
```

### UI/Interaction Checkpoints
- 删除确认弹窗样式与 Sidebar 保持一致
- 刷新按钮点击时有旋转动画反馈
- 暂停按钮在 streaming 时高亮/可点击，否则灰色/disabled
- 三个按钮 hover 效果保持现有样式

### General Checklist
- [ ] 不引入新的 Rust backend 代码（使用现有 IPC）
- [ ] 删除逻辑复用 App.tsx 中已有的 handlers
- [ ] 暂停功能使用 `runtime_session_stop` 现有命令
