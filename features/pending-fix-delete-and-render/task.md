# Tasks: fix-delete-and-render

## Task Breakdown

### 1. 删除功能 - Channel 删除
- [ ] 为 Channel 列表项的删除按钮添加 onClick 事件处理
- [ ] 添加删除确认对话框
- [ ] 删除成功后重置 activeChannel 状态（如果删除的是当前选中的）
- [ ] 刷新 channel 列表

### 2. 删除功能 - Thread 删除
- [ ] 为 Thread 列表项的删除按钮添加 onClick 事件处理
- [ ] 添加删除确认对话框
- [ ] 删除成功后关闭 ThreadPanel（如果删除的是当前活跃的 thread）
- [ ] 刷新 thread 列表

### 3. 删除功能 - Agent 删除
- [ ] 为 Agent 列表项的删除按钮添加 onClick 事件处理
- [ ] 添加删除确认对话框
- [ ] 删除成功后清理 selectedAgent 和相关状态
- [ ] 刷新 agent 列表

### 4. ThreadPanel 重选渲染修复
- [ ] 检查 handleThreadSelect 中 isThreadOpen 的设置逻辑
- [ ] 确保重选 thread 时 ThreadPanel 正确重新打开
- [ ] 确保重选时 activeThread 数据正确刷新

### 5. Channel → Agent → Thread 状态切换修复
- [ ] 梳理 App.tsx 中 channel/agent/thread 选择的完整状态流
- [ ] 修复 useEffect 依赖项导致的竞态问题
- [ ] 确保每次切换时状态清理和设置的时序正确
- [ ] 验证 MainContent 中 isChannelMode 切换逻辑

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 待开发 |
