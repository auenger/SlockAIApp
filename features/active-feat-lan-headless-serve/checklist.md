# Checklist: feat-lan-headless-serve

## Completion Checklist

### Development
- [ ] All tasks completed
- [ ] Code self-tested

### Code Quality
- [ ] CLI 参数解析清晰易用
- [ ] GUI 模式不受影响
- [ ] 复用 feat-lan-a2a-access 核心代码
- [ ] 信号处理安全（no UB on SIGINT）

### Testing
- [ ] Unit tests: CLI 参数解析
- [ ] Manual test: serve 启动/停止
- [ ] Manual test: A → B 连接（headless 模式）
- [ ] Tests passing

### Regression
- [ ] GUI 模式正常启动
- [ ] GUI 模式 LAN 功能正常（feat-lan-a2a-access）

### Documentation
- [ ] spec.md technical solution filled
