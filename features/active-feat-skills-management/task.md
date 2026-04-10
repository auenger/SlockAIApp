# Tasks: feat-skills-management

## Task Breakdown

### 1. 数据模型与存储
- [ ] 定义 Skill 数据模型（id, agent_id, name, type, config, status, created_at, updated_at）
- [ ] 设计 Skill 存储方案（JSONL 或 JSON，workspace 级别）
- [ ] 实现 Skill 存储的读写操作

### 2. Rust 后端 Commands
- [ ] `list_skills` — 列出指定 Agent 的所有 Skills
- [ ] `add_skill` — 为 Agent 添加新 Skill
- [ ] `update_skill` — 更新 Skill 配置
- [ ] `delete_skill` — 删除 Skill
- [ ] `get_skill_status` — 获取 Skill 运行状态

### 3. Frontend Types & IPC
- [ ] 扩展 `src/types.ts` — Skill 类型定义
- [ ] 扩展 `src/lib/ipc.ts` — Skill IPC commands
- [ ] 新增 `useSkills` hook

### 4. Frontend Skills 管理 UI
- [ ] Skills 列表组件
- [ ] Skill 添加/编辑表单
- [ ] Skill 删除确认
- [ ] Skill 状态指示器
- [ ] 集成到 Agent 详情页面或独立 tab

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 待开始实现 |
