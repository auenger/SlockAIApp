# Checklist: feat-token-usage-ui

## Completion Checklist

### Development
- [ ] All tasks completed
- [ ] Code self-tested (npm run dev 验证 UI 渲染)

### Code Quality
- [ ] TypeScript 类型无错误
- [ ] 样式使用 cn() + Tailwind，与项目一致
- [ ] TokenUsageBadge 不影响消息列表滚动性能

### Testing
- [ ] Vite build 无 TS 错误
- [ ] Channel 消息 token badge 正常显示
- [ ] Thread 消息 token badge 正常显示
- [ ] Agent Profile token 统计卡片正常显示
- [ ] 无 token 数据时不显示 badge（向后兼容）

### Documentation
- [ ] spec.md technical solution filled
