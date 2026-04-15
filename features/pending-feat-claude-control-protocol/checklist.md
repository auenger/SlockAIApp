# Checklist: feat-claude-control-protocol

## Completion Checklist

### Development
- [ ] Task 0: Control Protocol 消息格式验证完成
- [ ] Task 1: ExecuteParams +agent_id 变更完成
- [ ] Task 2: ProcessHandle + 进程池管理完成
- [ ] Task 3: stdout/stderr 解析逻辑复用完成
- [ ] Task 4: execute() 重写完成
- [ ] Task 5: 权限处理完成
- [ ] Task 6: 进程生命周期管理完成
- [ ] Task 7: 集成测试通过

### Code Quality
- [ ] Code style follows project conventions (log::info!, Rust idioms)
- [ ] No `--dangerously-skip-permissions` in production code
- [ ] Process cleanup on app exit (no orphan processes)
- [ ] Thread safety: Arc<Mutex<>> properly used for shared state

### Testing
- [ ] Single Agent multi-turn conversation works
- [ ] Multi-Agent concurrent execution works (process isolation)
- [ ] Permission handling works (if triggered)
- [ ] Crash recovery works (--resume)
- [ ] Idle timeout cleanup works
- [ ] Frontend zero-change verified

### Documentation
- [ ] spec.md technical solution updated with validated message format
- [ ] Control Protocol message format documented
