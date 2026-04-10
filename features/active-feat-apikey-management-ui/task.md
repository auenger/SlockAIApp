# Tasks: feat-apikey-management-ui

## Task Breakdown

### 1. Rust 后端 Commands
- [x] `list_api_keys` — 列出已存储的 API Key（脱敏）
- [x] `add_api_key` — 存储新的 API Key 到 keyring (existing `store_api_key`)
- [x] `delete_api_key` — 从 keyring 删除指定 Key (existing)
- [x] `verify_api_key` — 验证 Key 是否有效

### 2. Frontend Types & IPC
- [x] 扩展 `src/types.ts` — APIKey 类型定义（id, name, masked_key, provider, created_at）
- [x] 扩展 `src/lib/ipc.ts` — API Key IPC commands
- [x] 新增 `useApiKeys` hook

### 3. Frontend API Key 管理 UI
- [x] API Key 列表组件（脱敏显示）
- [x] 添加 Key 表单（Provider 选择 + Key 输入）
- [x] 删除确认弹窗
- [x] 集成到设置页面或 Modal

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 待开始实现，低优先级 |
| 2026-04-10 | All tasks implemented | Rust commands, types, IPC, hook, UI component, integrated into Sidebar |
