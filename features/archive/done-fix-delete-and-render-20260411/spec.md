# Feature: fix-delete-and-render 修复删除功能与渲染状态逻辑

## Basic Information
- **ID**: fix-delete-and-render
- **Name**: 修复删除功能与渲染状态逻辑
- **Priority**: 80
- **Size**: M
- **Dependencies**: None
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-10

## Description
修复三个紧密相关的 UI 层 bug：
1. channels、threads、agents 的删除按钮没有绑定事件处理函数，点击无反应
2. ThreadPanel 弹窗关闭后，再次选中 thread 不会重新渲染内容
3. 选中 channel 后切换到 agent 再选择 thread，无法正确渲染 agent 对话

后端 Tauri commands 和前端 hooks 已完整实现，问题集中在 UI 组件层的事件绑定和状态管理。

## User Value Points
1. **实体删除能力**: 用户可以删除不再需要的 channel、thread、agent，保持工作区整洁
2. **ThreadPanel 可靠渲染**: 关闭后重选 thread 能正确显示对话内容
3. **多视图切换一致性**: channel → agent → thread 切换后状态和内容正确同步

## Context Analysis

### Reference Code
- `src/App.tsx` - 主状态管理 (activeChannel, selectedAgent, activeThreadId, isThreadOpen)
- `src/components/MainContent.tsx` - 消息渲染，模式切换 (isChannelMode)
- `src/components/Sidebar.tsx` - 侧边栏，agent/channel/thread 列表
- `src/components/ThreadPanel.tsx` - Thread 弹窗面板
- `src/lib/ipc.ts` - IPC 调用 (deleteChannel, deleteThread, deleteAgent 已实现)
- `src/lib/useChannel.ts` - Channel hook (remove 方法已实现)
- `src/lib/useThreadChat.ts` - Thread hook (removeThread 方法已实现)

### Backend (已实现，无需修改)
- `src-tauri/src/commands/channel.rs` - delete_channel()
- `src-tauri/src/commands/thread.rs` - delete_thread()
- `src-tauri/src/commands/mod.rs` - delete_agent()

### Related Features
- feat-thread (Thread 对话系统)
- feat-channel (Channel 基础设施)

## Technical Solution

### Bug 1: 删除按钮无事件处理
**根因**: MainContent.tsx 和 Sidebar.tsx 中的 Trash2 图标按钮没有 onClick 事件处理
**修复**:
- 为 channel、thread、agent 的删除按钮添加 onClick handler
- 删除前添加确认对话框
- 删除成功后清理相关状态（如删除当前选中的 channel，需重置 activeChannel）
- 调用已有的 hooks 方法: `useChannel().remove()`, `useThreadChat().removeThread()`, IPC `deleteAgent()`

### Bug 2: ThreadPanel 关闭后重选不渲染
**根因**: 关闭 ThreadPanel 后 `isThreadOpen` 设为 false，重选 thread 时可能没有正确重新设置状态
**修复**:
- 确保 handleThreadSelect 始终设置 `isThreadOpen = true`
- 检查 activeThread 数据在重选时是否刷新
- 可能需要添加 key 或强制刷新机制确保 ThreadPanel 重新挂载

### Bug 3: Channel → Agent → Thread 状态切换失败
**根因**: 切换序列中状态清理和设置的时序问题，可能存在 useEffect 依赖导致状态被意外重置
**修复**:
- 梳理 channel/agent/thread 选择的完整状态流
- 确保每次选择时正确清理旧状态、设置新状态
- 检查 useEffect 依赖项，避免竞态条件

## Acceptance Criteria (Gherkin)

### User Story
作为一个用户，我希望能够删除不需要的实体，并且在切换不同视图时看到正确的对话内容。

### Scenarios

#### Scenario 1: 删除 Channel
```gherkin
Given 用户在侧边栏看到至少一个 channel
When 用户点击某个 channel 的删除按钮并确认
Then 该 channel 从列表中移除
And 如果删除的是当前选中的 channel，主内容区域清空
```

#### Scenario 2: 删除 Thread
```gherkin
Given 用户选中了一个 agent 并看到该 agent 的 thread 列表
When 用户点击某个 thread 的删除按钮并确认
Then 该 thread 从列表中移除
And 如果删除的是当前活跃的 thread，ThreadPanel 关闭
```

#### Scenario 3: 删除 Agent
```gherkin
Given 用户在侧边栏看到至少一个 agent
When 用户点击某个 agent 的删除按钮并确认
Then 该 agent 从列表中移除
And 如果删除的是当前选中的 agent，相关状态全部清理
```

#### Scenario 4: ThreadPanel 关闭后重选
```gherkin
Given 用户选中了一个 thread 并看到 ThreadPanel 展示对话内容
When 用户关闭 ThreadPanel
And 再次点击同一个 thread
Then ThreadPanel 重新打开并正确渲染该 thread 的对话内容
```

#### Scenario 5: 多视图切换
```gherkin
Given 用户选中了一个 channel 查看对话
When 用户切换选中一个 agent
And 然后选中该 agent 的一个 thread
Then ThreadPanel 正确展示该 thread 的对话内容（不是 channel 的内容）
```

#### Scenario 6: 删除取消
```gherkin
Given 用户点击了某个 channel/thread/agent 的删除按钮
When 用户在确认对话框中点击取消
Then 该实体不被删除，列表不变
```

### UI/Interaction Checkpoints
- 删除按钮 hover 时有视觉反馈
- 删除确认使用对话框而非 alert
- 删除成功后列表平滑更新
- ThreadPanel 打开/关闭有过渡动画

### General Checklist
- [x] 不修改后端代码（已实现）
- [x] 使用现有的 hooks 和 IPC 方法
- [x] 状态管理保持一致性

## Merge Record
- **Completed**: 2026-04-11
- **Branch**: feature/fix-delete-and-render
- **Merge Commit**: 8a8f15d
- **Archive Tag**: fix-delete-and-render-20260411
- **Conflicts**: none
- **Verification**: passed (6/6 Gherkin scenarios)
- **Stats**: 4 commits, 2 files changed (App.tsx, Sidebar.tsx)
