# Feature: feat-data-storage 数据存储架构（SQLite + JSONL 混合方案）

## Basic Information
- **ID**: feat-data-storage
- **Name**: 数据存储架构（SQLite + JSONL 混合方案）
- **Priority**: 80
- **Size**: L
- **Dependencies**: None
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-10

## Description

将项目从纯文件存储升级为 SQLite + JSONL 混合存储架构。SQLite 管理所有结构化元数据（agents, channels, threads, tasks, skills, activity log），JSONL 继续存储消息体内容。SQLite 只保存文件路径指针，不存储消息正文。

## User Value Points

1. **结构化查询能力** — Agent 列表、Channel 过滤、Task 看板、Activity 时间线等可通过 SQL 高效查询，不再需要扫描 JSON 文件
2. **Task 管理基础** — 为 Task 看板功能提供真正的数据库支持（状态过滤、排序、关联）
3. **数据一致性** — SQLite 事务保护元数据写入，避免并发场景下的数据丢失

## Storage Architecture

### SQLite Tables (结构化元数据)

| 表 | 字段 | 说明 |
|---|------|------|
| **agents** | id, name, emoji, avatar_path, enabled, runtime_type, description, created_at, updated_at | Agent 配置与状态 |
| **channels** | id, name, created_at, updated_at | Channel 元数据 |
| **channel_members** | channel_id, agent_id, role, joined_at | Channel ↔ Agent 多对多 |
| **threads** | id, agent_id, title, session_id, message_count, jsonl_path, created_at, updated_at | Thread 元数据 + JSONL 指针 |
| **tasks** | id, title, status, assignee, thread_id, description, created_at, updated_at | Task 看板 |
| **skills** | id, agent_id, name, skill_type, status, config_json, created_at, updated_at | 技能配置 |
| **activity_log** | id, timestamp, activity_type, agent_id, workspace_id, summary, details_json | Activity 时间线索引 |

### JSONL / Files (追加写入)

| 数据 | 存储位置 |
|------|---------|
| Thread 消息体 | `agents/{agent_id}/conversations/threads/{thread_id}.jsonl` |
| Channel 消息体 | 需从 Channel JSON 中拆分出来，独立 JSONL |
| Agent 身份文件 | `agents/{agent_id}/IDENTITY.md`, `SOUL.md` |
| Workspace 文件 | 代码、输出、上下文快照 |
| Memory 文件 | Agent 长期记忆 |

### 读写模式

```
写入消息:
  Frontend → Tauri IPC → Rust
    → JSONL.append(message)      # 消息体追加写入
    → SQLite.update(thread.last_message)  # 元数据更新

查询列表:
  Frontend → Tauri IPC → Rust
    → SQLite.query("SELECT * FROM threads WHERE agent_id = ?")
    → 返回轻量列表（不含消息体）

加载历史:
  Frontend → Tauri IPC → Rust
    → SQLite.query("SELECT jsonl_path FROM threads WHERE id = ?")
    → JSONL.read_all(path)
    → 返回完整消息列表
```

## Context Analysis

### Reference Code
- `src-tauri/src/storage/jsonl.rs` — 现有 JSONL 读写
- `src-tauri/src/storage/activity.rs` — 现有 Activity JSONL
- `src-tauri/src/workspace/` — Agent/Channel 管理
- `src/types.ts` — 前端类型定义

### Related Documents
- `project-context.md` — 需更新 "不使用 SQLite" 的设计决策

### Related Features
- 无前置依赖
- 后续 Task 看板功能、高级搜索功能依赖此 feature

## Technical Solution

### Phase 1: SQLite 基础设施
1. 添加 `rusqlite` + `refinery` (migration) 依赖到 Cargo.toml
2. 创建 `src-tauri/src/storage/db.rs` — 数据库初始化、连接池、migration 管理
3. 创建 `src-tauri/src/storage/migrations/` — SQL migration 文件
4. 在 Tauri app setup 时初始化数据库

### Phase 2: 数据迁移
5. 编写 migration 将现有 JSON 文件数据导入 SQLite
   - agents JSON → agents 表
   - channels JSON → channels + channel_members 表
   - threads JSON → threads 表
   - skills JSON → skills 表
6. 保持 JSONL 文件不变，SQLite 中记录 jsonl_path 指针

### Phase 3: API 层更新
7. 更新 Tauri Commands 使用 SQLite 查询
   - `list_agents` → `SELECT * FROM agents`
   - `list_channels` → `SELECT * FROM channels ORDER BY updated_at DESC`
   - `list_threads` → `SELECT * FROM threads WHERE agent_id = ?`
   - `list_tasks` → `SELECT * FROM tasks WHERE status = ?`
   - `list_activities` → `SELECT * FROM activity_log ORDER BY timestamp DESC LIMIT ?`
8. Channel 消息从 Channel JSON 中拆分到独立 JSONL 文件
9. 更新 `project-context.md` 设计决策

### 文件结构变更

```
src-tauri/src/storage/
├── db.rs                    # SQLite 初始化、连接管理
├── db_helpers.rs            # 通用查询辅助函数
├── migrations/
│   ├── V001__initial.sql    # 建表
│   └── V002__seed.sql       # 从现有数据迁移
├── jsonl.rs                 # JSONL 读写（保持不变）
├── activity.rs              # Activity 写入（改为双写：JSONL + SQLite）
└── keyring.rs               # API Key（保持不变）
```

## Acceptance Criteria (Gherkin)

### User Story
作为一个 AgentsZone 用户，我希望应用的数据存储高效且可靠，这样我可以在大量 Agent、Channel、Thread 中快速查找和管理。

### Scenarios (Given/When/Then)

#### Scenario 1: SQLite 数据库自动初始化
```gherkin
Given 应用首次启动
When Tauri app 初始化完成
Then SQLite 数据库文件在 workspace 目录下创建
And 所有表（agents, channels, threads, tasks, skills, activity_log）已创建
And migration 版本记录正确
```

#### Scenario 2: 现有数据自动迁移
```gherkin
Given 已有 Agent JSON 文件和 Channel JSON 文件存在
When 应用启动并检测到数据库为空
Then Agent 元数据从 JSON 文件迁移到 SQLite agents 表
And Channel 元数据从 JSON 文件迁移到 SQLite channels 表
And Thread 元数据迁移到 threads 表，jsonl_path 正确指向现有 JSONL 文件
And 迁移完成后原有 JSON 文件保留不删除（作为备份）
```

#### Scenario 3: Agent 列表查询
```gherkin
Given 数据库中有 5 个 Agent
When 前端调用 list_agents 命令
Then 返回 5 个 AgentSummary 记录
And 查询耗时 < 10ms（从 SQLite 读取，不扫描文件）
```

#### Scenario 4: Thread 消息写入
```gherkin
Given 用户在 Thread 中发送一条消息
When 消息写入完成
Then 消息体已追加到对应 JSONL 文件
And threads 表的 message_count 和 updated_at 已更新
And JSONL 文件路径与 threads 表 jsonl_path 一致
```

#### Scenario 5: Channel 消息独立存储
```gherkin
Given 一个 Channel 包含 100 条消息
When Channel 消息存储改为 JSONL 独立文件
Then Channel JSON 文件不再包含消息体，只保留元数据
And 消息体存储在 channels/{channel_id}/messages.jsonl
And channels 表记录 messages_jsonl_path 指针
```

#### Scenario 6: Task 看板查询
```gherkin
Given 数据库中有 TODO 3 个、IN PROGRESS 2 个、DONE 5 个
When 前端请求 Task 列表并过滤 status=TODO
Then 返回 3 个 TODO 状态的 Task
And 结果按 created_at 排序
```

### General Checklist
- [ ] SQLite 数据库文件位置合理（workspace 目录下，如 `workspaces/data/agentszone.db`）
- [ ] Migration 可重复执行（幂等）
- [ ] 现有数据迁移完成后功能不受影响
- [ ] JSONL 文件读写性能不下降
- [ ] project-context.md 已更新存储设计决策
- [x] `rusqlite` 使用 bundled feature（无需系统安装 SQLite）

## Merge Record
- **Completed**: 2026-04-10T16:47:00+08:00
- **Merged Branch**: feature/feat-data-storage
- **Merge Commit**: bec6542
- **Feature Commit**: 6963a5b
- **Archive Tag**: feat-data-storage-20260410
- **Conflicts**: None
- **Verification**: PASS (63/63 tests, 6/6 Gherkin scenarios with 1 deferred)
- **Files Changed**: 14 (5 new, 9 modified)
- **Duration**: ~1 hour (started 2026-04-10T17:05:00+08:00, completed same day)
