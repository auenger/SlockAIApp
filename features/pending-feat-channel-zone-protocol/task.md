# Tasks: feat-channel-zone-protocol

## Task Breakdown

### 1. Zone Agent Protocol 数据模型
- [ ] 创建 `src-tauri/src/context/zone_protocol.rs`
- [ ] 定义 `ChannelZoneProtocol` 和 `AgentMemberInfo` 结构体
- [ ] 实现 `from_channel()` 从 Channel + Agent 数据构建
- [ ] 实现 `render()` 渲染为 prompt 文本

### 2. ContextBuilder 扩展
- [ ] 在 `ContextBuilder` 中添加 `zone_protocol` 字段
- [ ] 添加 `with_zone_protocol()` builder 方法
- [ ] 修改 `build_context_prefix()` 按顺序注入 L2 层
- [ ] 确保 Thread 对话不注入 Zone Protocol

### 3. Channel 命令集成
- [ ] 修改 `send_channel_message` 构建 Zone Protocol
- [ ] 加载 Channel 成员的 Agent 身份信息
- [ ] 将 Zone Protocol 传入 ContextBuilder
- [ ] 验证完整 7 层组装顺序

### 4. 测试与验证
- [ ] 单元测试：Zone Protocol 渲染输出格式
- [ ] 单元测试：多 Agent Channel 场景
- [ ] 单元测试：单 Agent Channel 场景
- [ ] 手动测试：实际 Channel 对话验证

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 7层架构定义 + Zone Protocol 设计完成 |
