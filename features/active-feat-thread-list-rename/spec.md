# Feature: feat-thread-list-rename Thread 全局展示 & 重命名

## Basic Information
- **ID**: feat-thread-list-rename
- **Name**: Thread 全局展示 & 重命名
- **Priority**: 75
- **Size**: M
- **Dependencies**: none
- **Parent**: null
- **Children**: empty
- **Created**: 2026-04-11T23:00:00+08:00

## Description

改进 Thread 列表的展示方式：

1. **Threads 全局展示**：不再需要先选中 Agent 才能看到 Thread 列表。Sidebar 直接展示所有 Agent 的 Thread，每个 Thread 带有 Agent 标识（图标/名称），让用户一目了然。
2. **Thread 手动重命名**：用户可以手动修改 Thread 的显示名称，而不是使用系统自动生成的 "Thread with {agent.name}"。

## User Value Points

### VP1: Threads 全局展示
- 减少操作步骤：用户无需先选 Agent 再看 Thread
- 提高信息密度：所有对话集中展示，快速定位目标 Thread
- Agent 标识清晰：每个 Thread 通过 Agent 图标/名称明确归属

### VP2: Thread 重命名
- 个性化管理：用户可为 Thread 起有意义的名字（如"代码重构讨论"、"Bug 修复方案"）
- 快速辨识：相比自动生成的名字，自定义名称更易区分
- 双击/右键触发：简洁的交互方式

## Context Analysis

### Reference Code
- `src/components/Sidebar.tsx` (Lines 249-328) — Thread 列表渲染，当前按 selectedAgent 过滤
- `src/components/Sidebar.tsx` (Lines 169-176) — Agent 选中逻辑，选中 Agent 时清空 thread
- `src/components/ThreadPanel.tsx` (Lines 58-171) — Thread 对话面板，展示 title
- `src/types.ts` (Lines 67-92) — Thread / ThreadInfo 类型定义
- `src/lib/useThreadChat.ts` (Lines 89-137) — Thread 创建和列表加载（当前按 agent_id 过滤）
- `src/lib/ipc.ts` (Lines 182-228) — Thread IPC 封装
- `src-tauri/src/commands/thread.rs` — Rust 端 Thread commands
- `src-tauri/src/workspace/thread.rs` — Thread 数据模型

### Related Documents
- CLAUDE.md — 架构规则：Thread 最多一层嵌套

### Related Features
- feat-thread-chat (已完成) — Thread 1对1 对话基础
- feat-conversation-store (已完成) — 对话持久化

## Technical Solution

### Backend Changes

1. **`ThreadInfo` struct** (`workspace/thread.rs`): Extended with `agent_name`, `agent_emoji`, `agent_icon` fields (all optional via `#[serde(default)]`) to carry agent identity in the global list.

2. **`list_all_threads` command** (`commands/thread.rs`): New Tauri IPC command that queries all threads from SQLite via `db_helpers::list_all_threads()`, joins with agent info from the `agents` table, and returns `Vec<ThreadInfo>` sorted by `updated_at DESC`.

3. **`rename_thread` command** (`commands/thread.rs`): New Tauri IPC command accepting `thread_id` and `new_title`. Updates both the SQLite `threads` table and the thread's JSON file on disk. Returns updated `ThreadInfo`.

4. **Command registration** (`lib.rs`): Added `list_all_threads` and `rename_thread` to `invoke_handler`.

### Frontend Changes

5. **IPC layer** (`lib/ipc.ts`): Added `listAllThreads()` and `renameThread(threadId, newTitle)` wrappers.

6. **Types** (`types.ts`): Extended `ThreadInfo` with optional `agent_name`, `agent_emoji`, `agent_icon` fields.

7. **useThreadChat hook** (`lib/useThreadChat.ts`): Added `loadAllThreads()` and `renameThreadAction()` methods.

8. **App.tsx**: Changed from per-agent thread loading to global `loadAllThreads()` on mount. Thread selection auto-associates the correct agent. Thread deletion and creation work from global context.

9. **Sidebar.tsx**: Thread section now shows all threads with `AgentIcon` + agent name. "New Thread" button opens an agent picker when no agent is selected. Thread items support double-click inline rename (Enter/Escape/Blur).

10. **ThreadPanel.tsx**: Thread title in header supports double-click to rename inline.

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望在 Sidebar 中直接看到所有 Thread 列表（带 Agent 标识），并能手动修改 Thread 名称，以便更高效地管理和定位对话。

### Scenarios

#### VP1: Threads 全局展示

**Scenario 1: 展示全部 Threads（无需选择 Agent）**
```gherkin
Given 用户打开了应用
And 存在多个 Agent 的多个 Thread
When 用户查看 Sidebar 的 Thread 列表区域
Then 应该显示所有 Agent 的所有 Thread
And 每个 Thread 应显示对应的 Agent 标识（图标或名称）
And Thread 列表按最近更新时间排序
```

**Scenario 2: 点击 Thread 进入对话**
```gherkin
Given Sidebar 显示了全局 Thread 列表
When 用户点击某个 Thread
Then 该 Thread 打开并进入对话视图
And Thread 所属的 Agent 自动关联
```

**Scenario 3: 创建新 Thread 时选择 Agent**
```gherkin
Given 用户在全局 Thread 列表视图中
When 用户点击创建新 Thread
Then 用户需要选择目标 Agent
And 创建后新 Thread 出现在全局列表中
```

#### VP2: Thread 重命名

**Scenario 4: 双击 Thread 标题进入编辑**
```gherkin
Given Sidebar 显示了 Thread 列表
When 用户双击某个 Thread 的标题区域
Then 标题变为可编辑的输入框
And 输入框中显示当前标题
And 用户可以输入新名称
```

**Scenario 5: 确认重命名**
```gherkin
Given 用户正在编辑 Thread 标题
When 用户按下 Enter 键或点击输入框外部
Then Thread 标题更新为新名称
And 新名称持久化到后端存储
And Thread 列表刷新显示新名称
```

**Scenario 6: 取消重命名**
```gherkin
Given 用户正在编辑 Thread 标题
When 用户按下 Escape 键
Then 取消编辑，恢复原标题
And 输入框退出编辑模式
```

### UI/Interaction Checkpoints
- Thread 列表项增加 Agent 图标/名称标识
- Thread 标题支持双击进入编辑模式
- 编辑模式输入框样式：与列表视觉一致，有明确的编辑态反馈
- Agent 标识样式：小图标 + 名称缩写，不占用过多空间

### General Checklist
- [ ] 全局 Thread 列表按 updated_at 排序
- [ ] Thread 列表加载性能：使用 SQLite 轻量查询
- [ ] 重命名后端新增 `rename_thread` IPC command
- [ ] 重命名同时更新 SQLite 元数据和 JSON 文件
