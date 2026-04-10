# Tasks: feat-agent-a2a-trigger

## Task Breakdown

### 1. Agent 响应 @mention 解析
- [ ] 新增 `extract_agent_triggers()` 函数，复用 mention 解析
- [ ] 解析 Agent 回复内容中的 @{agent} 格式
- [ ] 过滤非 Channel 成员的 mention

### 2. 触发链执行引擎
- [ ] 定义 `TriggerContext` 结构体（depth, max_depth, triggered_agents）
- [ ] 实现 `execute_with_a2a()` 递归执行函数
- [ ] 深度限制检查（默认 max_depth=3）
- [ ] 去重检查（同触发链中同一 Agent 不重复触发）

### 3. Channel 命令集成
- [ ] 修改 `send_channel_message` 使用 A2A 执行引擎
- [ ] A2A 触发的消息附带触发来源信息
- [ ] A2A 触发的回复写入 Channel 对话记录

### 4. 前端 A2A 事件
- [ ] 新增 `agent://channel-a2a-start` 事件
- [ ] 前端区分用户触发和 Agent 触发
- [ ] A2A 触发在 UI 中有视觉区分

### 5. 测试与验证
- [ ] 单元测试：mention 解析（正常/异常）
- [ ] 单元测试：深度限制
- [ ] 单元测试：去重机制
- [ ] 手动测试：多 Agent 协作链

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | A2A 触发协议设计完成 |
