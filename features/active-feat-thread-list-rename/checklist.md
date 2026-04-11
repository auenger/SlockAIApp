# Checklist: feat-thread-list-rename

## Completion Checklist

### Development
- [x] All tasks completed
- [x] Code self-tested

### Code Quality
- [x] Code style follows conventions (cn() for styles, types in types.ts)
- [x] IPC 封装在 ipc.ts，hooks 在 use*.ts
- [x] Rust 端使用 log::info! / log::error! 日志

### Testing
- [x] 全局 Thread 列表正确加载所有 Agent 的 Thread
- [x] Thread 重命名持久化正确
- [x] 编辑模式交互流畅（Enter/Escape/Blur）

### Documentation
- [x] spec.md technical solution filled
