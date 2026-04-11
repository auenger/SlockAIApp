# Tasks: feat-thread-list-rename

## Task Breakdown

### 1. 后端：Thread 全局列表查询
- [ ] 新增 `list_all_threads` IPC command（查询所有 agent 的 thread，SQLite 轻量查询）
- [ ] 返回 ThreadInfo 列表，包含 agent_id、agent_name 等标识信息
- [ ] 按 updated_at 降序排序

### 2. 后端：Thread 重命名
- [ ] 新增 `rename_thread` IPC command（参数：thread_id, new_title）
- [ ] 更新 SQLite 元数据中的 title 字段
- [ ] 更新 Thread JSON 文件中的 title 字段
- [ ] 返回更新后的 ThreadInfo

### 3. 前端 IPC 层
- [ ] `src/lib/ipc.ts` 新增 `listAllThreads()` 和 `renameThread(id, title)` 封装

### 4. 前端：全局 Thread 列表 UI
- [ ] Sidebar Thread 区域改为加载所有 Thread（不再按 selectedAgent 过滤）
- [ ] Thread 列表项增加 Agent 标识（Agent 图标/名称）
- [ ] 点击 Thread 时自动关联 Agent
- [ ] 创建新 Thread 时提供 Agent 选择
- [ ] 列表按 updated_at 排序

### 5. 前端：Thread 重命名 UI
- [ ] Thread 列表项标题支持双击进入编辑模式
- [ ] 编辑态：inline input，Enter 确认，Escape 取消
- [ ] 点击外部区域（blur）确认保存
- [ ] 调用 `renameThread` IPC 持久化新名称
- [ ] ThreadPanel 标题区域也支持重命名

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
