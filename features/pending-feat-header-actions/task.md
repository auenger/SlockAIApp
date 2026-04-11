# Tasks: feat-header-actions

## Task Breakdown

### 1. MainContent Props 扩展
- [ ] 添加 `onDeleteChannel`、`onDeleteAgent`、`onRefresh`、`onStopSession` props
- [ ] 添加删除确认弹窗 state (`deleteConfirm`)

### 2. 删除按钮逻辑
- [ ] 根据 `isChannelMode` / `selectedAgent` 判断删除目标类型
- [ ] 实现确认弹窗组件（参考 Sidebar 模式）
- [ ] 绑定 onClick 到删除 handler
- [ ] 无选中项时按钮 disabled

### 3. 刷新按钮逻辑
- [ ] Channel 模式: 调用 `onRefresh` 重新加载 Channel 数据
- [ ] Agent/Thread 模式: 重新加载 Thread 消息
- [ ] 添加刷新中旋转动画状态

### 4. 暂停按钮逻辑
- [ ] 绑定 `onStopSession` 到 onClick
- [ ] 根据 `channelIsStreaming` 或 runtime 状态控制按钮 enabled/disabled
- [ ] 暂停成功后清理 streaming 状态

### 5. App.tsx 传递 Props
- [ ] 从 App.tsx 传入 `handleDeleteChannel`、`handleDeleteAgent`
- [ ] 实现 refresh handler（根据当前视图调用对应 reload）
- [ ] 传入 stopSession handler

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-11 | Feature created | 待开发 |
