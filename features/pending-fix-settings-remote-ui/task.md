# Tasks: fix-settings-remote-ui

## Task Breakdown

### 1. Card 样式统一（VP1）
- [ ] RemoteConnectionsPanel: ConnectionCard 使用 brutal-border / brutal-shadow 替代 gray-700 边框
- [ ] Card 背景改为 bg-brutal-bg 或白色，提升文字可读性
- [ ] StatusBadge 使用 brutalist 配色（bg-brutal-green, gray-400, bg-brutal-pink, bg-brutal-yellow）
- [ ] 操作按钮（Test/Edit/Delete/Add）改为 brutal-btn 风格
- [ ] "Add Connection" 表单区域对齐 brutalist 风格
- [ ] "Edit Connection" 区域对齐 brutalist 风格
- [ ] "Delete Confirm" 区域对齐 brutalist 风格
- [ ] 空状态提示对齐 brutalist 风格

### 2. Emoji → SVG 图标（VP2）
- [ ] BridgeWorkspacePanel: 📁 → lucide-react Folder 图标
- [ ] BridgeWorkspacePanel: 📄 → lucide-react FileText 图标
- [ ] Agent 卡片中 emoji → AgentIcon 组件或对应 SVG
- [ ] 确认所有 emoji 已替换完毕

### 3. Workspace 溢出修复（VP3）
- [ ] BridgeWorkspacePanel 添加折叠/展开切换按钮
- [ ] 设置 BridgeWorkspacePanel 最大高度约束
- [ ] 确保文件列表在 max-height 内 overflow-y-auto 滚动
- [ ] 折叠时隐藏 workspace 内容，展开时恢复

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
