# Tasks: feat-agent-edit

## Task Breakdown

### 1. 后端 - updateAgent 命令
- [x] Rust 端新增 `update_agent` command
- [x] 支持更新 name, icon, emoji, creature, vibe 等字段
- [x] 更新 agent identity 配置文件
- [x] 添加 icon 字段到 AgentIdentity 和 AgentSummary

### 2. 前端 IPC 层
- [x] 在 ipc.ts 中新增 `updateAgent` 函数
- [x] 定义 UpdateAgentRequest 类型
- [x] 调用后端 update_agent command

### 3. EditAgentModal 组件
- [x] 创建 EditAgentModal 组件（基于 CreateAgentModal 结构）
- [x] 预填现有 Agent 属性（从 getAgentIdentity 加载）
- [x] 集成 IconPicker 组件（来自 feat-svg-icon-system）
- [x] 表单验证（name 必填）
- [x] 调用 updateAgent IPC 保存
- [x] 只发送变更字段

### 4. 编辑入口 - Agent Profile 页
- [x] 在 Agent Profile 页添加编辑按钮（Pencil 图标）
- [x] 点击后打开 EditAgentModal
- [x] 保存后自动刷新 Profile 数据

### 5. 编辑入口 - Sidebar
- [x] 在 Sidebar Agent 项添加编辑入口（hover 显示 Pencil 按钮）
- [x] 点击后打开 EditAgentModal
- [x] 保存后自动刷新 Agent 列表

### 6. UI 状态同步
- [x] 编辑保存后更新全局 Agent 列表状态（通过 useAgentStatus.scan）
- [x] selectedAgent 与 allAgents 自动同步（App.tsx useEffect）
- [x] Sidebar、Channel、Thread、Profile 即时反映修改

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 待开发，依赖 feat-svg-icon-system |
| 2026-04-10 | Implementation complete | 全部 6 个任务完成，Rust + TS 编译通过 |
