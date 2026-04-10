# Tasks: feat-agent-profile-page

## Task Breakdown

### 1. Profile 数据加载
- [x] 创建 `useAgentProfile` hook 或在 MainContent 中加载 Identity 数据
- [x] 调用 `getAgentIdentity(agentId)` 获取身份信息
- [x] 调用 `getAgentContext(agentId)` 获取 context 信息（含 role 描述）
- [x] 处理加载状态和错误

### 2. Profile UI 重构
- [x] 替换硬编码的 Agent 头部区域（名称、emoji）
- [x] 替换 Role section，显示真实 Identity 内容
- [x] 替换 Configuration section，显示真实 Runtime 信息
- [x] 添加 Workspace 路径显示
- [x] 处理无 Agent 选中的空状态
- [x] 保持 brutal-border 风格一致

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-09 | Feature created | 等待开发 |
| 2026-04-09 | Implementation complete | useAgentProfile hook, PROFILE tab real data |
