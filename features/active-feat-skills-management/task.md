# Tasks: feat-skills-management

## Task Breakdown

### 1. 数据模型与存储
- [x] 定义 Skill 数据模型（id, agent_id, name, type, config, status, created_at, updated_at）
- [x] 设计 Skill 存储方案（JSON 文件，workspace 级别，agents/{agent_id}/skills/skills.json）
- [x] 实现 Skill 存储的读写操作（SkillStore: load_all, add, update, delete, get）

### 2. Rust 后端 Commands
- [x] `list_skills` — 列出指定 Agent 的所有 Skills
- [x] `add_skill` — 为 Agent 添加新 Skill
- [x] `update_skill` — 更新 Skill 配置
- [x] `delete_skill` — 删除 Skill
- [x] `get_skill_status` — 获取 Skill 运行状态

### 3. Frontend Types & IPC
- [x] 扩展 `src/types.ts` — Skill 类型定义（SkillInfo, SkillType, SkillStatus）
- [x] 扩展 `src/lib/ipc.ts` — Skill IPC commands（listSkills, addSkill, updateSkill, deleteSkill, getSkillStatus）
- [x] 新增 `useSkills` hook（含 mock 数据 fallback）

### 4. Frontend Skills 管理 UI
- [x] Skills 列表组件（集成到 MainContent.tsx SKILLS tab）
- [x] Skill 添加/编辑表单（SkillFormModal 组件）
- [x] Skill 删除确认（内联确认按钮）
- [x] Skill 状态指示器（Active/Inactive/Connecting/Error 状态标签）
- [x] 集成到 MainContent SKILLS tab，使用真实数据

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 待开始实现 |
| 2026-04-10 | 全部任务完成 | Rust 后端 Skill CRUD + 前端完整 UI |
