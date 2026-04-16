# Checklist: feat-task-async-exec

## Completion Checklist

### Development
- [ ] All tasks completed
- [ ] Code self-tested

### Code Quality
- [ ] 不阻塞 UI 线程
- [ ] 线程安全（Arc<Mutex> 正确使用）
- [ ] Agent busy 状态正确管理

### Testing
- [ ] Async 执行成功：Task → 后台 Runtime → 完成
- [ ] 有 Channel 时结果投递到 Channel
- [ ] 无 Channel 时结果在 TaskDetail 中查看
- [ ] 执行失败重试逻辑
- [ ] 重试耗尽标记为 blocked
- [ ] Cancel 功能正常
- [ ] 不影响 Realtime 模式

### Documentation
- [ ] spec.md technical solution filled
