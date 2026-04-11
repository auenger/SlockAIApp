# Checklist: feat-thread-list-rename

## Completion Checklist

### Development
- [ ] All tasks completed
- [ ] Code self-tested

### Code Quality
- [ ] Code style follows conventions (cn() for styles, types in types.ts)
- [ ] IPC 封装在 ipc.ts，hooks 在 use*.ts
- [ ] Rust 端使用 log::info! / log::error! 日志

### Testing
- [ ] 全局 Thread 列表正确加载所有 Agent 的 Thread
- [ ] Thread 重命名持久化正确
- [ ] 编辑模式交互流畅（Enter/Escape/Blur）

### Documentation
- [ ] spec.md technical solution filled
