# Tasks: feat-workspace-open-finder

## Task Breakdown

### 1. Rust 后端
- [x] 在 `src-tauri/src/commands/mod.rs` 新增 `open_in_finder` command
- [x] 在 `src-tauri/src/lib.rs` 注册新 command

### 2. 前端 IPC
- [x] 在 `src/lib/ipc.ts` 新增 `openWorkspaceInFinder(agentId)` 函数

### 3. 前端 UI
- [x] 在 `src/components/MainContent.tsx` 路径栏添加「在 Finder 中打开」按钮
- [x] 引入 `FolderOpen` 图标
- [x] 添加点击处理逻辑

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-11 | All tasks completed | Rust open_in_finder command (cross-platform macOS/Windows/Linux), IPC wrapper, FolderOpen button in path bar |
