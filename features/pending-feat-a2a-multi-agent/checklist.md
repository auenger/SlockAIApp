# Checklist: feat-a2a-multi-agent

## Completion Checklist

### Development
- [ ] All tasks in task.md completed
- [ ] Code self-tested (cargo build + npm run build + npm run dev)
- [ ] No new compiler warnings

### Code Quality
- [ ] Delegation logic 幂等（重复发送不创建重复任务）
- [ ] Push notification 幂等处理（相同 event_id 不重复处理）
- [ ] Artifact store 文件清理策略（过期清理）
- [ ] 所有新 IPC command 有 proper error handling

### Testing
- [ ] Push Notification 单元测试（event parsing, signature verification）
- [ ] Delegation engine 单元测试（状态转换覆盖所有路径）
- [ ] Artifact store 单元测试（CRUD + consumer tracking）
- [ ] Integration test: 完整委托流程（mock 2 agents）
- [ ] Frontend component rendering tests
- [ ] All tests passing

### Security
- [ ] Webhook signature verification enabled by default
- [ ] Push URL validation（防止 SSRF）
- [ ] Artifact access control（只能访问自己参与的 task 的 artifacts）
- [ ] Delegation authorization（不能冒充其他 Agent 发起委托）

### Performance
- [ ] Push notification handler 低延迟（< 100ms p99）
- [ ] Artifact store 大文件支持（streaming read/write）
- [ ] Delegation context summary generation 不阻塞主线程
- [ ] 前端协作视图渲染性能（大量节点时不卡顿）

### Documentation
- [ ] spec.md technical solution filled
- [ ] Collaboration protocol flow documented
- [ ] Push notification security model documented
