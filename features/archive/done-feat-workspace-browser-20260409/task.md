# Tasks: feat-workspace-browser

## Task Breakdown

### 1. 后端 Workspace 浏览命令
- [x] 新增 `list_workspace_dir(agent_id, subpath?)` 命令
- [x] 新增 `read_workspace_file(agent_id, file_path)` 命令
- [x] 定义 DirectoryEntry 和 FileContent 数据结构
- [x] 路径安全校验（防止 `../` 路径穿越）

### 2. 前端 Workspace Tab 重构
- [x] 创建 `useWorkspace` hook
- [x] 替换硬编码目录树为动态加载
- [x] 实现文件点击 → 内容加载
- [x] 实现真实路径展示
- [x] 空状态处理

### 3. IPC 集成
- [x] 添加 Workspace IPC 函数到 ipc.ts
- [x] 添加类型到 types.ts

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-09 | Feature created | 等待开发 |
| 2026-04-09 | Implementation complete | 后端命令、前端hook、IPC集成均已完成 |
