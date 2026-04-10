# Checklist: fix-delete-and-render

## Completion Checklist

### Development
- [ ] 所有 tasks 完成
- [ ] 代码自测通过

### Code Quality
- [ ] 代码风格遵循项目规范
- [ ] 使用现有 hooks 和 IPC 方法，不重复实现
- [ ] 状态管理逻辑清晰，无竞态条件

### Testing
- [ ] 删除 channel 功能验证
- [ ] 删除 thread 功能验证
- [ ] 删除 agent 功能验证
- [ ] ThreadPanel 关闭后重选验证
- [ ] Channel → Agent → Thread 切换验证
- [ ] 删除取消操作验证

### Documentation
- [ ] spec.md 技术方案已填写
