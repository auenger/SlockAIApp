# Tasks: feat-task-channel-selector

## Task Breakdown

### 1. TaskCreateModal — Channel 下拉选择器
- [x] 加载 Channel 列表：组件内调用 `listChannels()` 获取 channels
- [x] 替换 Channel 文本输入为 `<select>` 下拉，显示 channel name
- [x] 处理 channelId prop 传入时的预选逻辑（从 channels 列表中匹配）
- [x] 允许清空 Channel 选择（选择 "None" option）

### 2. TaskCreateModal — Agent 智能过滤
- [x] 当 Channel 选中时，从 channel.members 提取 agent_id 列表
- [x] 过滤 agent 下拉列表，只显示 Channel 中的 Agent
- [x] Channel 切换时：如果当前 Agent 不在新 Channel 中，重置选择
- [x] Channel 切换时：如果新 Channel 只有 1 个 Agent，自动选中
- [x] 无 Channel 时恢复显示所有 Agent

### 3. 编辑模式适配
- [x] 编辑已有 Task 时正确回填 Channel 和 Agent
- [x] 编辑模式下 Channel 下拉与 Agent 过滤逻辑一致

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-16 | All tasks completed | Channel dropdown + agent filtering + edit mode |
