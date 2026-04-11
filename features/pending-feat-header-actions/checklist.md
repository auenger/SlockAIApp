# Checklist: feat-header-actions

## Completion Checklist

### Development
- [ ] All tasks completed
- [ ] Code self-tested (手动测试删除/刷新/暂停三个按钮)
- [ ] 删除逻辑与 Sidebar 已有逻辑保持一致
- [ ] 刷新按钮在 Channel 和 Agent 模式下均可用
- [ ] 暂停按钮仅在 streaming 时可用

### Code Quality
- [ ] Code style follows conventions (cn() for styles, TypeScript types)
- [ ] No unnecessary new files created (复用现有组件模式)
- [ ] Props interface 清晰，无冗余参数

### Testing
- [ ] 手动测试 Channel 删除流程
- [ ] 手动测试 Agent 删除流程
- [ ] 手动测试 Channel 刷新
- [ ] 手动测试 Thread 刷新
- [ ] 手动测试暂停正在执行的 Agent

### Documentation
- [ ] spec.md technical solution filled
