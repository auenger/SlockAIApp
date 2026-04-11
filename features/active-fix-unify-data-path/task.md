# Tasks: fix-unify-data-path

## Task Breakdown

### 1. 核心路径修改
- [x] 修改 `src-tauri/src/lib.rs` 中 `resolve_workspace_root()` 函数，使用 `~/.agentszone/` 替代 `app_data_dir() + "workspaces"`
- [x] 移除或更新 `DEFAULT_WORKSPACE_DIR` 常量

### 2. 验证与清理
- [x] 确认日志输出显示正确的新路径
- [x] 检查前端 mock 路径是否需要同步更新（`src/lib/useWorkspace.ts`、`src/lib/useAgentProfile.ts`）

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-11 | Feature created | 等待开发 |
| 2026-04-11 | Implementation done | lib.rs 改用 dirs::home_dir()/.agentszone，前端 mock 路径已同步，93 测试全通过 |
