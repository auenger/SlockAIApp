# Checklist: feat-agent-runtime-ui

## Completion Checklist

### Development
- [x] All tasks completed
- [x] Code self-tested
- [x] CreateAgentModal 正确传递 runtime_type

### Code Quality
- [x] 使用 cn() 合并样式 (used tailwind template literals matching existing pattern)
- [x] 类型定义集中在 types.ts
- [x] IPC 调用走统一 ipc.ts 封装

### Testing
- [x] 验证 Claude Code 可用时 UI 正确显示
- [x] 验证 Codex 未安装时显示安装提示
- [x] 验证创建 agent 后 config 包含正确 runtime_type

### Documentation
- [x] spec.md technical solution filled
- [x] Runtime 选择器交互说明

## Verification Record
- **Date**: 2026-04-10T19:15:00+08:00
- **Status**: PASSED
- **Results**: All 17 tasks completed, 4/4 Gherkin scenarios passed, 0 TypeScript errors, Vite build successful
- **Evidence**: `features/active-feat-agent-runtime-ui/evidence/verification-report.md`
