# Feature: fix-agent-create-bugs Agent 创建流程修复

## Basic Information
- **ID**: fix-agent-create-bugs
- **Name**: Agent 创建流程修复（Icon 保存 + 列表刷新）
- **Priority**: 90
- **Size**: S
- **Dependencies**: none
- **Parent**: null
- **Children**: none
- **Created**: 2026-04-11T20:30:00+08:00

## Description

新增 Agent 时存在两个 bug：
1. **Icon 未正确保存**: 用户在创建 Agent 时选择了 icon，但创建完成后 icon 没有被持久化。根因是 Rust 端 `CreateAgentRequest` 结构体缺少 `icon` 字段，前端传递了 icon 但后端完全忽略了该字段。
2. **Agent 列表不自动刷新**: 创建 Agent 完成后，Sidebar 中的 Agent 列表不会自动更新，需要 reload 整个 App 才能看到新创建的 Agent。根因是 `scan` 函数（`useAgentStatus` hook）只调用 `getAgentRuntimeStatus`，不会从 workspace manager 重新加载 agent 列表。

## User Value Points

1. **Icon 正确保存**: 用户在创建 Agent 时选择的 icon 能被正确持久化到后端存储，后续查看 Agent 时能正确显示选定的 icon
2. **创建后自动刷新**: 创建 Agent 成功后，Sidebar 的 Agent 列表自动更新显示新 Agent，无需手动 reload

## Context Analysis

### Reference Code

**Bug 1 - Icon 未保存:**
- `src/components/CreateAgentModal.tsx` — 前端正确设置了 icon state 并传递
- `src/lib/ipc.ts:99-101` — IPC 层正确传递 icon 参数
- `src-tauri/src/commands/mod.rs` — **BUG**: `CreateAgentRequest` 结构体缺少 `icon` 字段
- `src-tauri/src/workspace/manager.rs` — `create_agent` 方法签名不接受 icon 参数
- `src-tauri/src/workspace/agent.rs` — Agent 数据结构需确认 icon 字段处理

**Bug 2 - 列表不刷新:**
- `src/components/Sidebar.tsx:455` — `onSuccess={scan}` 传递给 CreateAgentModal
- `src/lib/useAgentStatus.ts:96-118` — **BUG**: `scan` 函数只获取 runtime status，不重新加载 agent 列表
- `src-tauri/src/commands/mod.rs` — `get_agent_runtime_status` 命令可能需要修改

### Related Features
- feat-agent-create-ui (Agent 创建 UI)
- feat-svg-icon-system (SVG Icon 系统)
- feat-agent-edit (Agent 编辑能力)

## Technical Solution

### Fix 1: Icon 保存
1. 在 Rust `CreateAgentRequest` 结构体中添加 `icon: Option<String>` 字段
2. 更新 `manager.rs` 中的 `create_agent` 方法签名，添加 icon 参数
3. 在创建 Agent 时将 icon 写入 identity 配置
4. 确保 `AgentIdentity` 结构体正确序列化/反序列化 icon 字段
5. 确认 EditAgentModal 的 icon 编辑流程也正常工作

### Fix 2: 列表刷新
1. 在 `useAgentStatus` hook 的 `scan` 函数中，增加对 `list_agents` 或类似命令的调用
2. 或者在 Rust 端的 `get_agent_runtime_status` 中先 reload agents 再返回状态
3. 确保 `scan` 被调用后 Sidebar 的 agent 列表 state 被正确更新

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望创建 Agent 时选择的 icon 能正确保存，并且创建完成后 Agent 列表能自动刷新，这样我就不需要手动 reload 整个应用。

### Scenarios

#### Scenario 1: Icon 正确保存
```gherkin
Given 用户打开了创建 Agent 对话框
When 用户填写名称、选择 icon、选择 runtime 类型并点击创建
Then 新创建的 Agent 的 icon 应该被正确保存到后端存储
And 在 Sidebar 中查看该 Agent 时能正确显示选定的 icon
And 重新打开 App 后该 Agent 的 icon 仍然正确显示
```

#### Scenario 2: Agent 列表自动刷新
```gherkin
Given 用户打开了创建 Agent 对话框
When 用户填写完信息并成功创建 Agent
Then Sidebar 中的 Agent 列表应该立即自动更新
And 新创建的 Agent 应该出现在列表中
And 不需要手动 reload 整个 App
```

#### Scenario 3: 不选择 Icon 时的默认行为
```gherkin
Given 用户打开了创建 Agent 对话框
When 用户填写名称但未选择 icon 就点击创建
Then Agent 应该使用默认 icon 或 emoji 作为头像
And 创建成功后列表正常刷新
```

### UI/Interaction Checkpoints
- 创建成功后 Sidebar 立即显示新 Agent
- 新 Agent 的 icon 与创建时选择的一致
- 无需 reload 即可对新 Agent 进行操作

### General Checklist
- [ ] Rust 端 `CreateAgentRequest` 包含 icon 字段
- [ ] 创建流程完整传递 icon 到存储层
- [ ] `scan` 函数能正确刷新 agent 列表
- [ ] 不影响已有的 Agent 编辑功能
