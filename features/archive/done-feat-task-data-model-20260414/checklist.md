# Checklist: feat-agent-task-system

## Completion Checklist

### Development
- [ ] All 5 sub-features completed (子2和子3可并行)
- [x] Code self-tested
- [x] Task CRUD verified through IPC
- [x] cancel_task command 可用
- [ ] Kanban drag-and-drop working (@dnd-kit)
- [ ] 搜索和过滤功能正常
- [ ] 多选批量操作可用
- [ ] Real-time task execution tested (注入 channel.rs 流程)
- [ ] Async queue dispatch tested (后台 poll 线程)
- [ ] Task 取消机制 tested
- [ ] Task 失败重试 tested (MAX_RETRY=2)
- [ ] Conversation → <task-suggestions> 解析 tested
- [ ] TaskSuggestionCard 交互 tested (确认/编辑/忽略)
- [ ] Sub-task + dependency tested
- [x] 循环依赖检测 tested (would_create_cycle)
- [ ] 父子任务级联 tested
- [ ] A2A task passing tested (复用 task-suggestions 协议)

### Code Quality
- [x] Code style follows conventions (cn(), types.ts, log::info!)
- [x] Rust commands registered in lib.rs
- [x] TypeScript types centralized in types.ts
- [x] Task.id 统一为 string (UUID)，与 DB TEXT 一致
- [x] creatorId 始终填充（user 或 agent）
- [x] IPC calls wrapped in ipc.ts
- [x] Hooks follow use*.ts naming pattern
- [x] No API keys hardcoded
- [x] UTF-8 safe string operations
- [x] DB 表有索引和 CHECK 约束
- [x] 无不可用的 FK 约束 (agents/channels 引用已移除)

### Testing
- [ ] Task CRUD unit tests
- [ ] Task status transition tests
- [ ] parse_task_suggestions 容错 tests (无 tag / JSON 错误 / 字段缺失)
- [ ] would_create_cycle 循环依赖 tests
- [ ] Dependency resolution tests
- [ ] 父子任务级联状态 tests
- [ ] TaskEngine cancel/retry tests
- [ ] Frontend component tests (if applicable)
- [ ] Tests passing

### Documentation
- [ ] spec.md technical solution filled (含跨切面补充)
- [ ] DB migration files documented (V002__tasks_v2.sql)
- [x] IPC command documentation updated (含 cancel_task)
- [ ] Task Suggestion Protocol 文档化

## Verification Record

| Date | Status | Details |
|------|--------|---------|
| 2026-04-14 | PASS | 12/12 tasks complete, cargo check pass, tsc pass, 11 IPC commands verified |
| | | Evidence: features/active-feat-task-data-model/evidence/verification-report.md |
