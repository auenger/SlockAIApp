# Tasks: fix-agent-create-bugs

## Task Breakdown

### 1. Rust 后端 - Icon 字段支持
- [x] 在 `CreateAgentRequest` 结构体添加 `icon: Option<String>` 字段
- [x] 更新 `create_agent` 方法签名添加 icon 参数
- [x] 在 Agent 创建流程中将 icon 写入 identity/存储
- [x] 确认 `AgentIdentity` 序列化包含 icon 字段

### 2. 前端 - Agent 列表刷新
- [x] 修复 `useAgentStatus` hook 的 `scan` 函数，确保重新加载 agent 列表
- [x] 验证 `Sidebar` 中 `onSuccess` 回调正确触发列表刷新
- [x] 确认创建成功后新 Agent 立即可见

### 3. 验证
- [x] 创建新 Agent 并选择 icon，确认 icon 被正确保存
- [x] 创建新 Agent 后确认列表自动刷新
- [x] 不选择 icon 创建 Agent，确认默认行为正常
- [x] 编辑 Agent 时 icon 功能不受影响

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-11 | Feature created | 分析完成，两个 bug 根因已明确 |
