# Checklist: feat-lan-a2a-access

## Completion Checklist

### Development
- [ ] All tasks completed
- [ ] Code self-tested

### Code Quality
- [ ] Code style follows conventions (cn(), types.ts, log::info!)
- [ ] TCP server 不阻塞 UI 线程
- [ ] 错误处理完备（端口冲突、连接断开）
- [ ] 资源管理（线程清理、连接释放）

### Testing
- [ ] Unit tests: run_adapter_server_loop
- [ ] Unit tests: handle_tcp_connection with real AdapterServer
- [ ] Unit tests: start/stop lifecycle
- [ ] Integration test: A → B 连接（手动验证）
- [ ] Tests passing

### Regression
- [ ] 现有本地 Runtime 功能正常
- [ ] 现有 A2A Client 功能正常
- [ ] 现有远程连接管理 UI 正常

### Documentation
- [ ] spec.md technical solution filled
- [ ] CLAUDE.md updated if needed
