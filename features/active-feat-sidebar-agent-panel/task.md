# Tasks: feat-sidebar-agent-panel

## Task Breakdown

### 1. MessageSquare 按钮交互状态
- [ ] 为 MessageSquare 按钮添加 activeTab 状态绑定
- [ ] 添加 onClick 处理切换 activeTab 为 'CHAT'
- [ ] 添加 active 高亮样式

### 2. ActiveAgentPanel 组件
- [ ] 创建 ActiveAgentPanel 弹出组件
- [ ] 从 useAgentStatus 获取 agents 并过滤 available 状态
- [ ] 渲染 Agent 列表（复用 AgentIcon）
- [ ] 实现 click-away 和 ESC 关闭逻辑
- [ ] 空状态提示

### 3. 对话入口集成
- [ ] 面板中 Agent 点击触发 onAgentSelect
- [ ] 点击后关闭面板

### 4. 样式
- [ ] brutal design 风格（brutal-border、brutal-shadow）
- [ ] 绝对定位从按钮下方展开
- [ ] hover 和 active 交互反馈

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-17 | Feature created | 初始化 |
