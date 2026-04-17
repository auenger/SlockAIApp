# Feature: feat-remote-agent-model 远程 Agent 代理模型

## Basic Information
- **ID**: feat-remote-agent-model
- **Name**: 远程 Agent 代理模型（后端拉取 + 映射 + 状态同步）
- **Priority**: 70
- **Size**: S
- **Dependencies**: feat-lan-a2a-bridge, feat-a2a-remote-client
- **Parent**: feat-remote-agent-integration
- **Children**: []
- **Created**: 2026-04-17

## Description
建立远程 Agent 代理模型：从已连接的远程 bridge 自动拉取 agent 列表，在本地创建代理 Agent 实体（connection_mode=remote），并保持健康状态同步。这是远程 Agent 融入的基础层，后续 UI 和消息执行都依赖此模型。

## User Value Points
1. **远程 Agent 自动发现** — 连接远程 bridge 后，自动获取该 workspace 上的所有 agents
2. **统一 Agent 实体** — 远程 agents 映射为与本地 agents 相同的 AgentSummary 结构

## Context Analysis
### Reference Code
- `src-tauri/src/runtime/a2a/remote.rs` — RemoteConnectionManager，已有 `get_agents()` 能力
- `src-tauri/src/runtime/a2a/remote_runtime.rs` — RemoteRuntime 已有基础框架
- `src-tauri/src/workspace/manager.rs` — AgentManager，管理 agent 列表
- `src-tauri/src/workspace/agent.rs` — 单个 agent workspace
- `src-tauri/src/workspace/identity.rs` — AgentIdentity 身份定义
- `src-tauri/src/storage/db.rs` — SQLite 存储
- `src/types.ts` — `AgentSummary` 已有 `connection_mode: ConnectionMode` 字段

### Key Insight
`AgentSummary` 类型已定义 `ConnectionMode`:
```typescript
type ConnectionMode = "local" | { remote: { connection_id: string } };
```
Rust 端已有对应的 `RemoteConnectionInfo`。基础设施已就绪，需要的是**填充逻辑**。

## Technical Solution

### 方案：Bridge Agent Sync Service

#### 1. 新增 Tauri Commands
```rust
// src-tauri/src/commands/remote_connection.rs

// 从指定远程连接拉取 agents 并创建本地代理
#[tauri::command]
async fn sync_remote_agents(connection_id: String) -> Result<Vec<AgentSummary>, String>

// 获取所有远程代理 agents（跨所有连接）
#[tauri::command]
async fn get_remote_agents() -> Result<Vec<AgentSummary>, String>

// 刷新单个远程连接的 agents
#[tauri::command]
async fn refresh_remote_agents(connection_id: String) -> Result<(), String>
```

#### 2. Agent Sync 逻辑
- 调用 `bridge.getAgents` 获取远程 agent 列表
- 为每个远程 agent 创建本地代理记录（存储在 SQLite agents 表中）
- `connection_mode` 设为 `{ remote: { connection_id } }`
- `workspace_path` 指向远程路径（虚拟路径或远程标识）
- 不创建本地 workspace 目录

#### 3. 状态同步
- 连接健康检查时同步更新远程 agent 状态
- 连接断开时标记远程 agents 为 offline
- 连接恢复时重新同步

#### 4. IPC 接口（前端）
```typescript
// src/lib/ipc.ts 新增
export function syncRemoteAgents(connectionId: string): Promise<AgentSummary[]>;
export function getRemoteAgents(): Promise<AgentSummary[]>;
export function refreshRemoteAgents(connectionId: string): Promise<void>;
```

## Acceptance Criteria (Gherkin)
### User Story
作为用户，当我连接到远程 workspace 后，我希望系统能自动发现远程 agents 并将它们添加到我的 agent 列表中。

### Scenarios
```gherkin
Scenario: 远程连接成功后自动同步 agents
  Given 用户已添加一个远程连接且状态为 online
  When 用户点击"同步 Agents"按钮或连接健康检查通过
  Then 系统调用 bridge.getAgents 获取远程 agent 列表
  And 每个远程 agent 在本地创建代理记录
  And 这些 agents 的 connection_mode 为 { remote: { connection_id } }
  And 返回的 AgentSummary 列表包含所有远程 agents

Scenario: 远程连接断开时 agents 状态更新
  Given 有 3 个远程 agents 来自 connection-1
  When connection-1 健康检查失败（状态变为 offline）
  Then 这 3 个远程 agents 的状态标记为 offline
  And 前端可通过 agent 状态感知连接变化

Scenario: 重复同步不产生重复 agents
  Given connection-1 已同步过 2 个远程 agents
  When 用户再次同步 connection-1
  Then 已存在的 agents 更新信息而非重复创建
  And 新出现的 agents 被添加
  And 消失的 agents 被标记为不可用

Scenario: 删除远程连接时清理关联 agents
  Given connection-1 有 2 个关联远程 agents
  When 用户删除 connection-1
  Then 关联的 2 个远程 agents 被清理
  And 这些 agents 从 channel 成员列表中移除
```

### General Checklist
- [x] 不影响本地 agents 的任何功能
- [x] 远程 agent 数据持久化到 SQLite
- [x] 并发同步安全（同一连接不重复同步）

## Merge Record
- **Completed**: 2026-04-17T09:45:00+08:00
- **Merged Branch**: feature/feat-remote-agent-model
- **Merge Commit**: 7bab0bb
- **Archive Tag**: feat-remote-agent-model-20260417
- **Conflicts**: None
- **Verification**: 270 tests passed, 4/4 Gherkin scenarios passed
- **Stats**: 1 commit, 7 files changed, 489 lines added, 3 deleted
