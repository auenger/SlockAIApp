# Tasks: feat-agent-a2a-trigger

## Task Breakdown

### 1. Agent 响应 @mention 解析
- [x] 新增 `extract_agent_triggers()` 函数，复用 mention 解析
- [x] 解析 Agent 回复内容中的 @{agent} 格式
- [x] 过滤非 Channel 成员的 mention

### 2. 触发链执行引擎
- [x] 定义 `TriggerContext` 结构体（depth, max_depth, triggered_agents）
- [x] 实现 `extract_valid_triggers()` 和 `classify_triggers()` 函数
- [x] 深度限制检查（默认 max_depth=3）
- [x] 去重检查（同触发链中同一 Agent 不重复触发）

### 3. Channel 命令集成
- [x] 重构 `send_channel_message` 使用队列驱动的 A2A 执行引擎
- [x] A2A 触发的消息附带触发来源信息（triggered_by, depth）
- [x] A2A 触发的回复写入 Channel 对话记录

### 4. 前端 A2A 事件
- [x] 新增 `agent://channel-a2a-start` 事件
- [x] 新增 `agent://channel-a2a-depth-exceeded` 事件
- [x] 前端区分用户触发和 Agent 触发（is_a2a, triggered_by, a2a_depth）
- [x] AgentStreamState 扩展 A2A 元数据字段

### 5. 测试与验证
- [x] 单元测试：mention 解析（正常/异常）— 7 个新测试用例
- [x] 单元测试：深度限制 — TriggerContext + extract_valid_triggers 测试
- [x] 单元测试：去重机制 — classify_triggers dedup 测试
- [x] 手动测试：编译通过（Rust + TypeScript），93 个测试全部通过

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | A2A 触发协议设计完成 |
| 2026-04-11 | Implementation complete | 全部 5 个 task 完成，93 测试通过 |
