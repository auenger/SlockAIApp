# Feature: feat-remote-agent-ui 远程 Agent UI 融入

## Basic Information
- **ID**: feat-remote-agent-ui
- **Name**: 远程 Agent UI 融入（Sidebar + Channel 成员 + Thread + 视觉区分）
- **Priority**: 70
- **Size**: S
- **Dependencies**: feat-remote-agent-model
- **Parent**: feat-remote-agent-integration
- **Children**: []
- **Created**: 2026-04-17

## Description
将远程 agents 无缝融入现有 UI 体系：Sidebar 展示远程 agents（带连接来源标识）、Channel 成员选择器支持添加远程 agents、Thread 列表支持远程 agent 对话入口。远程与本地 agents 有清晰视觉区分。

## User Value Points
1. **远程 Agent 可见** — 用户在 Sidebar 和各处 agent 列表中能看到远程 agents
2. **远程 Agent 可加入 Channel** — 在 Channel 设置中能将远程 agent 添加为成员
3. **远程 Agent 可选为 Thread 对话对象** — 在 Thread 列表中可选择远程 agent 进行 1:1 对话

## Context Analysis
### Reference Code
- `src/components/Sidebar.tsx` — 侧边栏 Agent 列表
- `src/components/MainContent.tsx` — Channel 成员管理
- `src/components/ThreadPanel.tsx` — Thread 面板
- `src/lib/useAgents.ts` — Agent 列表 hook（如果存在）
- `src/lib/useChannel.ts` — Channel 管理 hook
- `src/types.ts` — AgentSummary 类型

### Design Consideration
远程 agents 需要在 UI 上与本地 agents 有明确区分：
- 远程 agent 名称旁显示连接来源标签
- 使用不同的视觉标识（如云朵 icon、remote badge）
- offline 状态有明确的视觉提示

## Technical Solution

### 1. Sidebar 远程 Agent 展示
- Sidebar 的 AGENTS 区域同时展示本地和远程 agents
- 远程 agents 通过小标签或 icon 区分来源（如显示连接名称）
- 按 connection 分组显示或混合排列（混合更自然）
- offline 的远程 agents 灰色显示

### 2. Channel 成员选择器
- 现有 Channel 成员添加 UI 扩展支持远程 agents
- Agent 选择列表合并展示本地 + 远程 agents
- 远程 agents 带 remote 标识
- 添加远程 agent 到 channel 时验证连接状态

### 3. Thread Agent 选择
- Thread 面板的 agent 选择器包含远程 agents
- 选择远程 agent 时提示"远程对话"模式
- offline 远程 agent 不可选或提示连接状态

### 4. 前端组件改造
```typescript
// 通用 AgentBadge 组件（用于各处展示）
interface AgentBadgeProps {
  agent: AgentSummary;
  showConnectionBadge?: boolean; // 显示远程连接标识
  showStatus?: boolean;          // 显示在线状态
}

// Agent 选择器（用于 Channel 成员添加、Thread 对话选择）
interface AgentSelectorProps {
  agents: AgentSummary[];        // 合并后的本地+远程 agents
  selectedIds: string[];
  onSelectionChange: (ids: string[]) => void;
  filter?: 'all' | 'local' | 'remote';
}
```

## Acceptance Criteria (Gherkin)
### User Story
作为用户，我希望在 UI 的各处都能看到和使用远程 agents，就像使用本地 agents 一样自然。

### Scenarios
```gherkin
Scenario: Sidebar 展示远程 agents
  Given 用户有一个 online 的远程连接，该连接有 2 个 agents
  When 用户打开 Sidebar
  Then AGENTS 区域显示本地和远程 agents
  And 远程 agents 有视觉区分（连接标识或 remote badge）
  And 远程 agents 的在线状态与连接状态一致

Scenario: 添加远程 agent 到 Channel
  Given 用户在一个 Channel 中
  And 有一个 online 的远程连接的 agent "RemoteHelper"
  When 用户点击"添加成员"
  Then 成员选择列表包含 "RemoteHelper"（带 remote 标识）
  When 用户选择 "RemoteHelper" 并确认
  Then "RemoteHelper" 出现在 Channel 成员列表中
  And Channel header 显示远程 agent 成员

Scenario: 选择远程 agent 进行 Thread 对话
  Given 有一个 online 的远程 agent "RemoteHelper"
  When 用户在 Thread 面板选择 agent
  Then agent 列表包含 "RemoteHelper"
  When 用户选择 "RemoteHelper"
  Then Thread 面板切换为与 RemoteHelper 对话模式

Scenario: 远程 agent 离线时 UI 反馈
  Given Channel 中有一个远程 agent 成员
  When 该远程连接断开
  Then 该远程 agent 在成员列表中显示为 offline
  And 用户 @mention 该 agent 时提示"Agent 当前不可用"
```

### UI/Interaction Checkpoints
- Sidebar agent 列表：本地/远程 agents 混合展示，远程有标识
- Channel 成员添加：选择器包含远程 agents，带筛选能力
- Thread agent 选择：远程 agents 可选，offline 状态提示
- 远程 agent badge：统一的视觉语言（颜色/icon/标签）

### General Checklist
- [ ] 远程/本地 agent 视觉区分清晰但不过度
- [ ] offline 远程 agents 不影响 UI 性能
- [ ] 无障碍性（screen reader 可区分远程/本地）
