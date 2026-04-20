# Tasks: feat-sidebar-agent-panel

## Task Breakdown

### 1. MessageSquare 按钮交互状态
- [x] 为 MessageSquare 按钮添加 activeTab 状态绑定
- [x] 添加 onClick 处理切换 activeTab 为 'CHAT'
- [x] 添加 active 高亮样式

### 2. ActiveAgentPanel 组件
- [x] 创建 ActiveAgentPanel 弹出组件
- [x] 从 useAgentStatus 获取 agents 并过滤 available 状态
- [x] 渲染 Agent 列表（复用 AgentIcon）
- [x] 实现 click-away 和 ESC 关闭逻辑
- [x] 空状态提示

### 3. 对话入口集成
- [x] 面板中 Agent 点击触发 onAgentSelect
- [x] 点击后关闭面板

### 4. 样式
- [x] brutal design 风格（brutal-border、brutal-shadow）
- [x] 绝对定位从按钮下方展开
- [x] hover 和 active 交互反馈

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-17 | Feature created | 初始化 |
| 2026-04-20 | Implementation complete | ActiveAgentPanel 组件 + Sidebar 集成 |
