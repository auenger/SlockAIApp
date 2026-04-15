# Tasks: feat-task-ui-board

## Feature Info
- **ID**: feat-task-ui-board
- **Name**: Task 看板 + 列表 + 详情 UI（可与执行引擎并行）
- **Priority**: 85
- **Parent**: feat-agent-task-system
- **Size**: S

## Task List

### Phase 1: 基础组件

- [x] 安装 @dnd-kit 拖拽库
- [x] 新建 src/components/task/ 目录
- [x] TaskStatusBadge.tsx — 状态徽章组件
- [x] TaskCard.tsx — 任务卡片组件
- [x] TaskCreateModal.tsx — 创建/编辑对话框
- [x] TaskAssignDropdown.tsx — Agent 分配下拉

### Phase 2: 视图组件

- [x] TaskBoard.tsx — Kanban 看板 (@dnd-kit 拖拽)
- [x] TaskList.tsx — 列表视图 (含多选批量操作)
- [x] TaskDetail.tsx — 详情侧边抽屉

### Phase 3: 导航集成

- [x] Sidebar 集成 TASKS 导航入口 (含未完成数红点)
- [x] MainContent TASKS tab 改造为 Channel Task 视图

### Phase 4: 搜索过滤

- [x] 全局视图切换 (Board / List)
- [x] 搜索栏 + 过滤下拉 (status/assignee/priority/channel)

## Acceptance Criteria

可以手动创建 Task、看板展示、拖拽改状态、查看详情、搜索过滤。

## Progress Log

| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-15 | Feature started | Branch + worktree created |
| 2026-04-15 | Implementation complete | All task components implemented, TS errors in useTaskEngine fixed |
