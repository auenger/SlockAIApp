# Feature: feat-agent-workspace-persist Agent Workspace 验证与修复

## Basic Information
- **ID**: feat-agent-workspace-persist
- **Name**: Agent Workspace 创建验证与修复
- **Priority**: 75
- **Size**: S
- **Dependencies**: none
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-10T23:30:00+08:00

## Description

用户反馈创建 Agent 后找不到对应的 workspace 文件夹和 MD 文件。经排查：

1. Agent workspace 目录实际创建在 Tauri app data 目录 (`~/Library/Application Support/{bundle_id}/workspaces/`)，而非项目根目录
2. 代码中 `initialize_workspace()` 和 `create_agent_internal()` 的逻辑完整，会创建 IDENTITY.md、SOUL.md 和子目录
3. 需要验证实际运行时是否正确执行，以及前端是否能正确获取 workspace 状态

**范围说明**：Channel 对话不写入 per-agent JSONL。Channel 是共享上下文容器，对话记录保存在 `channels/channel_{id}.json` 中；Thread 是 1:1 对话，已有 per-agent JSONL。两者语义不同，无需将 Channel 消息复制到 agent workspace。

## User Value Points

### VP1: Agent workspace 创建可靠性
确保每个 Agent 创建时，workspace 目录结构完整（IDENTITY.md、SOUL.md、conversations/、context/、output/）。

### VP2: Workspace 状态可观测
用户能通过 UI 或状态接口确认 workspace 的实际路径和文件完整性。

## Context Analysis

### Reference Code
- `src-tauri/src/lib.rs` — `resolve_workspace_root()` 使用 `app.path().app_data_dir()`，`initialize_workspace()` 在 setup 中调用
- `src-tauri/src/workspace/manager.rs` — `AgentManager`、`initialize_workspace()`、`create_agent_internal()`
- `src-tauri/src/workspace/agent.rs` — `AgentWorkspace::initialize()` 创建子目录
- `src-tauri/src/workspace/identity.rs` — `write_to_file()` 写入 IDENTITY.md
- `src-tauri/src/workspace/templates.rs` — `sync_agent()` 写入 SOUL.md
- `src-tauri/src/commands/mod.rs` — `get_workspace_status` 命令
- `src-tauri/src/storage/jsonl.rs` — Thread JSONL 存储引擎（已工作正常）

### Related Documents
- project-context.md — Workspace = Agent 文件目录

### Related Features
- feat-agent-workspace-design (已完成) — Workspace 设计
- feat-data-storage (已完成) — 数据存储架构

## Technical Solution

### 需要排查/修复的问题

#### 1. 验证 workspace 创建流程
- 确认 `initialize_workspace()` 在 app 启动时被调用
- 确认 `create_agent_internal()` 在用户创建 Agent 时被调用
- 检查错误是否被静默吞掉（`if let Err(e) = ...` 只有 warn log）

#### 2. 检查 workspace 路径返回
- `get_workspace_status` 命令是否正确返回 workspace_root 路径
- 前端是否能显示 workspace 实际路径，方便用户定位文件

#### 3. 确保创建流程健壮性
- 如果 workspace 目录被意外删除，需要有恢复机制
- `load()` 方法在目录不存在时直接返回空，不会重建

### 修改范围

可能涉及的文件：
- `src-tauri/src/workspace/manager.rs` — load() 时可能需要触发 workspace 重建
- `src-tauri/src/commands/mod.rs` — `get_workspace_status` 是否返回足够信息
- `src/lib/ipc.ts` — 前端是否正确显示 workspace 路径

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望创建 Agent 后能确认其 workspace 目录和文件已正确创建。

### Scenarios (Given/When/Then)

#### Scenario 1: App 启动创建 default workspace
```gherkin
Given 首次启动应用
When 初始化完成
Then workspaces/agents/default/ 目录存在
And IDENTITY.md 文件存在
And SOUL.md 文件存在
And conversations/ 子目录存在
```

#### Scenario 2: 创建 Agent 时 workspace 完整
```gherkin
Given 应用已启动
When 用户创建名为 "Claude" 的 Agent
Then workspaces/agents/claude/ 目录存在
And IDENTITY.md 包含正确的 name、emoji、runtime_type
And SOUL.md 包含个性化人设
And conversations/threads/ 目录存在
```

#### Scenario 3: workspace 状态可查询
```gherkin
Given 已创建多个 Agent
When 调用 get_workspace_status
Then 返回 workspace_root 实际路径
And 返回 agent 数量
And 返回 active_agent_id
```

### General Checklist
- [x] workspace 创建流程可靠
- [x] 错误不会静默丢失
- [x] workspace 路径对用户可见

## Merge Record

- **Completed**: 2026-04-11T13:00:00+08:00
- **Merged Branch**: feature/feat-agent-workspace-persist
- **Merge Commit**: f7dd795
- **Archive Tag**: feat-agent-workspace-persist-20260411
- **Conflicts**: none
- **Verification**: passed (63/63 tests, 3/3 Gherkin scenarios)
- **Development Stats**: 3 commits, 6 files changed
