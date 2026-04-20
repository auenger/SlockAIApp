# Tasks: feat-remote-sidebar-panel

## Task Breakdown

### 1. Monitor 按钮切换逻辑
- [x] 为 Monitor 按钮添加 toggle 状态（展开/折叠远程面板）
- [x] 面板展开时高亮 Monitor 按钮状态
- [x] 面板折叠/展开动画

### 2. 远程概览面板组件
- [x] 创建 RemoteOverviewPanel 组件
- [x] 渲染远程连接列表（名称 + 状态灯 + Agent 数量）
- [x] 每个连接可折叠展开 Agent 子列表
- [x] Agent 条目显示名称 + 状态 + 远程标记

### 3. 数据集成
- [x] 复用 useRemoteConnections hook 获取连接列表
- [x] 复用 useAllAgents / useRemoteAgents 获取远程 Agent
- [x] 连接健康状态展示（healthy/unhealthy/unknown）
- [x] 空状态处理（无远程连接时的提示）

### 4. 样式与交互
- [x] Neo-Brutalism 风格适配
- [x] 状态指示灯颜色（绿/红/灰）
- [x] 空状态引导文案

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-20 | All tasks completed | RemoteOverviewPanel component created, Monitor toggle added to Sidebar, data integration via existing hooks |
