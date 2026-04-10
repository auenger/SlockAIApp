# Tasks: feat-thread-panel-live

## Task Breakdown

### 1. ThreadPanel 重构
- [x] 重写 ThreadPanel.tsx，移除所有硬编码内容
- [x] 定义新 Props 接口: thread, agent, onSend, onClose
- [x] 实现消息列表渲染（参考 MainContent 消息渲染）
- [x] 实现消息输入框和发送按钮
- [x] 实现空状态提示

### 2. App.tsx 集成
- [x] 传递真实 thread/agent 数据到 ThreadPanel
- [x] 连接 onSend 到 useThreadChat.send
- [x] 处理 ThreadPanel 与 MainContent 的交互

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-09 | Feature created | 等待开发 |
| 2026-04-09 | Implemented | ThreadPanel 重构完成，App.tsx 集成完成 |
