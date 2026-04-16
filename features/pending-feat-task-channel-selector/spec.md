# Feature: feat-task-channel-selector — Task Channel 选择器 + Agent 过滤

## Basic Information
- **ID**: feat-task-channel-selector
- **Name**: Task Channel 选择器 + Agent 智能过滤
- **Priority**: 70
- **Size**: S
- **Dependencies**: none
- **Parent**: feat-task-exec-runtime
- **Children**: none
- **Created**: 2026-04-16

## Description

优化 Task 创建流程中的 Channel 选择体验：

1. **Channel 下拉选择器**：替换当前的文本输入框，改为从已有 Channel 列表中选择
2. **Agent 智能过滤**：选择 Channel 后，根据 Channel-Agent 绑定关系自动过滤可选 Agent
3. **Agent 自动选择**：如果 Channel 只有一个 Agent，自动选中；如果之前选择的 Agent 在新 Channel 中存在，保持选择

## User Value Points

1. **Channel 快速选择**：用户从已有 Channel 列表中选取，避免手动复制 Channel ID
2. **Agent-Channel 一致性**：确保选择的 Agent 与 Channel 的成员关系一致，避免执行时出错

## Context Analysis

### Reference Code
- `src/components/task/TaskCreateModal.tsx` — 当前 Channel 文本输入 + Agent 选择
- `src/lib/ipc.ts` — `listChannels()`, `getChannel()` 已有 IPC 函数
- `src/types.ts` — `Channel`, `ChannelMember`, `AgentWithRuntime` 类型定义
- `src/components/task/TaskView.tsx` — TaskView 传入 agents 和 channelId props

### Related Documents
- 已完成 `feat-task-ui-board` — Task 看板 UI 基础
- 已完成 `feat-channel-infra` — Channel 基础设施，包含成员管理

### Related Features
- `feat-task-realtime-exec` (依赖本 feature)
- `feat-task-async-exec` (依赖本 feature)

## Technical Solution

### 方案概述

1. **TaskCreateModal 改造**
   - 加载 Channel 列表 (`listChannels()`)
   - Channel 字段改为 `<select>` 下拉，显示 Channel 名称
   - 选择 Channel 后加载 Channel members
   - Agent 下拉根据 channel members 过滤

2. **Agent 过滤逻辑**
   ```
   if (selectedChannel) {
     availableAgents = agents.filter(a => channelMembers.includes(a.agent.agent_id))
     if (availableAgents.length === 1) auto-select
     if (currentAgent not in availableAgents) reset selection
   } else {
     availableAgents = all agents  // 无 Channel 时显示全部
   }
   ```

3. **数据流**
   - 组件 mount 时 `listChannels()` 加载 channels
   - Channel 变更时从 channel.members 提取 agent IDs
   - 不需要额外 IPC 调用，Channel 对象已包含 members

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我想要在创建 Task 时从下拉列表选择 Channel 和 Agent，以便快速绑定正确的执行上下文。

### Scenarios

#### Scenario 1: Channel 下拉选择
```gherkin
Given 系统中存在 Channel "项目讨论" (members: Agent-A, Agent-B) 和 Channel "代码审查" (members: Agent-C)
When 用户打开 Task 创建 Modal
Then Channel 字段显示为下拉选择器
And 下拉列表包含 "项目讨论" 和 "代码审查"
And 默认显示 "Select channel..."
```

#### Scenario 2: Agent 按 Channel 过滤
```gherkin
Given 用户选择了 Channel "项目讨论" (members: Agent-A, Agent-B)
And 系统中共有 Agent-A, Agent-B, Agent-C
When 查看 Agent 下拉列表
Then 只显示 Agent-A 和 Agent-B
And Agent-C 不在列表中
```

#### Scenario 3: Channel 切换时 Agent 自动调整
```gherkin
Given 用户已选择 Channel "项目讨论" 和 Agent-A
When 用户切换 Channel 为 "代码审查" (members: Agent-C)
Then Agent 选择重置为空
And Agent 下拉只显示 Agent-C
```

#### Scenario 4: 单 Agent Channel 自动选择
```gherkin
Given 用户选择了 Channel "单人任务" (仅包含 Agent-A)
When Agent 列表刷新
Then Agent-A 自动被选中
```

#### Scenario 5: 无 Channel 时显示所有 Agent
```gherkin
Given 用户未选择任何 Channel
When 查看 Agent 下拉列表
Then 显示系统中所有 Agent
```

### UI/Interaction Checkpoints
- Channel 下拉使用 `brutal-border` 风格，与现有 UI 一致
- 显示 Channel 名称，value 为 channel.id
- 可选显示 Channel 中的 Agent 数量作为提示

### General Checklist
- 不破坏已有 channelId prop 传入时的自动绑定逻辑
- 保持 `executionMode` 选择不受影响
- 编辑模式（TaskCreateModal with task prop）正确回填 Channel 和 Agent
