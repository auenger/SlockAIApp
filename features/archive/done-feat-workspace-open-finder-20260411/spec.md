# Feature: feat-workspace-open-finder Workspace 在 Finder 中打开

## Basic Information
- **ID**: feat-workspace-open-finder
- **Name**: Workspace 在 Finder 中打开
- **Priority**: 50
- **Size**: S
- **Dependencies**: none
- **Parent**: null
- **Children**: none
- **Created**: 2026-04-11

## Description
在 Workspace 浏览器的路径导航栏中增加「在 Finder 中打开」按钮，点击后使用系统文件管理器打开当前 agent 的 workspace 目录。

## User Value Points
1. **快速访问 workspace 文件** — 用户无需手动查找 `~/.agentszone/agents/{agent-id}/` 路径，一键在 Finder 中打开

## Context Analysis

### Reference Code
- `src/components/MainContent.tsx` (lines 677-700) — Workspace 路径栏，已有回退、刷新、复制按钮
- `src/lib/useWorkspace.ts` — Workspace hook，管理路径和加载逻辑
- `src/lib/ipc.ts` — IPC 封装
- `src-tauri/src/commands/mod.rs` — 已有 workspace 相关 commands

### Related Documents
- project-context.md — Workspace 目录结构说明

### Related Features
- feat-workspace-browser (已完成) — Workspace 文件浏览器基础

## Technical Solution

### Rust 端
1. 在 `src-tauri/src/commands/mod.rs` 新增 `open_in_finder` command：
   - 接收 `agent_id` 参数
   - 解析为 `~/.agentszone/agents/{agent_id}/` 路径
   - 使用 Tauri 的 `tauri::api::shell::open` 或 Rust 的 `std::process::Command` 调用 `open` (macOS) / `explorer` (Windows)
   - 优先使用 Tauri 内置 API 以保证跨平台

### 前端
1. 在 `src/lib/ipc.ts` 新增 `openWorkspaceInFinder(agentId: string)` IPC 调用
2. 在 `src/components/MainContent.tsx` 的路径栏中添加一个 FolderOpen 图标按钮（在复制按钮旁）
3. 点击时调用 IPC，用 `open` 命令打开当前 agent 的 workspace 目录

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我想要在浏览 agent workspace 时一键在系统文件管理器中打开对应文件夹，以便快速管理文件。

### Scenarios (Given/When/Then)

**Scenario 1: 正常打开 Finder**
```gherkin
Given 用户已选择一个 agent
And 用户在 WORKSPACE 标签页
When 用户点击路径栏中的「在 Finder 中打开」按钮
Then 系统文件管理器应打开该 agent 的 workspace 目录
```

**Scenario 2: 未选择 agent 时按钮不可用**
```gherkin
Given 用户未选择任何 agent
When 用户查看 WORKSPACE 标签页
Then 「在 Finder 中打开」按钮应不可见或禁用
```

**Scenario 3: 路径不存在时的处理**
```gherkin
Given 用户已选择一个 agent
But 该 agent 的 workspace 目录不存在
When 用户点击「在 Finder 中打开」按钮
Then 应显示错误提示 "Workspace directory not found"
```

### UI/Interaction Checkpoints
- 按钮使用 `FolderOpen` 图标 (lucide-react)
- 按钮风格与现有路径栏按钮一致 (brutal-border + hover 效果)
- 按钮位于复制按钮旁边

### General Checklist
- [x] 跨平台支持 (macOS Finder / Windows Explorer)
- [x] 错误处理 (目录不存在)
- [x] 与现有 UI 风格一致

## Merge Record

- **Completed**: 2026-04-11T21:30:00+08:00
- **Merged Branch**: feature/feat-workspace-open-finder
- **Merge Commit**: 95fa578
- **Feature Commit**: e7a9aaf
- **Archive Tag**: feat-workspace-open-finder-20260411
- **Conflicts**: none
- **Verification**: passed (3/3 Gherkin scenarios)
- **Stats**: 1 commit, 4 files changed, 73 insertions
