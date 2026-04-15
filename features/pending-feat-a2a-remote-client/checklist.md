# Checklist: feat-a2a-remote-client

## Completion Checklist

### Development
- [ ] All tasks in task.md completed
- [ ] Code self-tested (cargo build + npm run build + npm run dev)
- [ ] No new compiler warnings

### Code Quality
- [ ] Auth tokens never logged or exposed in error messages
- [ ] TLS 配置安全（skip-cert 默认关闭）
- [ ] Frontend components follow existing cn() pattern
- [ ] IPC error handling consistent with existing patterns

### Testing
- [ ] Backend: RemoteConnection CRUD 单元测试
- [ ] Backend: Auth module 单元测试
- [ ] Backend: Manager health check 定时器测试
- [ ] Frontend: Panel 渲染测试（组件挂载）
- [ ] Integration: 端到端远程对话 mock 测试
- [ ] All tests passing

### Security
- [ ] Tokens stored in Keyring, not plaintext DB
- [ ] No token leakage in logs or responses
- [ ] HTTPS enforced for remote endpoints (configurable override)
- [ ] Input validation on endpoint URLs (prevent SSRF)

### Documentation
- [ ] spec.md technical solution filled
- [ ] Security model documented
