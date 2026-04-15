# Checklist: feat-a2a-adapter

## Completion Checklist

### Development
- [ ] All tasks in task.md completed
- [ ] Code self-tested (cargo build + cargo test)
- [ ] No new compiler warnings

### Code Quality
- [ ] Adapter 模式清晰，不侵入原有 claude.rs / codex.rs 代码
- [ ] Error handling 一致（A2A Error 格式）
- [ ] 资源管理正确（socket 清理、进程回收）

### Testing
- [ ] Adapter 单元测试（mock CLI output → A2A Message 转换）
- [ ] Server handler 集成测试（HTTP request → response）
- [ ] Unix socket 通信测试
- [ ] Agent 生命周期集成测试（start → send → stop）
- [ ] All tests passing

### Regression
- [ ] 现有 Channel 对话功能正常（不走 A2A 时行为不变）
- [ ] 现有 Thread 对话功能正常
- [ ] Agent 创建/编辑流程不受影响

### Documentation
- [ ] spec.md technical solution filled
- [ ] Adapter 设计决策文档化
