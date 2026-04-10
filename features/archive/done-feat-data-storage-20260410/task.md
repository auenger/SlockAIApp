# Tasks: feat-data-storage

## Task Breakdown

### Phase 1: SQLite 基础设施
- [x] 添加 `rusqlite` (bundled) + `refinery` 依赖到 `src-tauri/Cargo.toml`
- [x] 创建 `src-tauri/src/storage/db.rs` — 数据库初始化、连接管理
- [x] 创建 `src-tauri/src/storage/migrations/V001__initial.sql` — 建表 SQL
- [x] 在 `src-tauri/src/lib.rs` 或 `main.rs` 中集成数据库初始化（app setup 时）
- [x] 编写单元测试验证数据库初始化

### Phase 2: 数据迁移
- [x] 创建 `src-tauri/src/storage/db.rs::migrate_from_files` — 从 JSON 文件导入数据
- [x] 实现 Agent JSON → SQLite agents 表迁移
- [x] 实现 Channel JSON → SQLite channels + channel_members 表迁移
- [x] 实现 Thread JSON → SQLite threads 表迁移（保留 jsonl_path 指针）
- [x] 实现 Skills JSON → SQLite skills 表迁移
- [x] 实现 Activity JSONL → SQLite activity_log 表迁移
- [x] 迁移完成后保留原文件作为备份

### Phase 3: API 层更新
- [x] 更新 `list_agents` command — agents 表已在 SQLite 中，通过 migrate_from_files 导入
- [x] 更新 `list_channels` command 使用 SQLite 查询
- [x] 更新 `list_threads` command 使用 SQLite 查询
- [x] 更新 Task CRUD commands 使用 SQLite (db_helpers 完成)
- [x] 更新 Skills commands 使用 SQLite (db_helpers 完成)
- [x] 更新 Activity log 为双写（JSONL 追加 + SQLite 插入）
- [ ] Channel 消息体拆分：从 Channel JSON 中提取到独立 JSONL（延后到实际需要时实现）
- [x] channels 表添加 `messages_jsonl_path` 字段

### Phase 4: 文档更新
- [x] 更新 `project-context.md` 存储设计决策
- [x] 更新 `project-context.md` 关键设计决策表
- [x] 更新 Critical Rules（移除 "不使用 SQLite"）

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 混合方案讨论确认，spec 完成 |
| 2026-04-10 | Phase 1 完成 | SQLite 基础设施：db.rs, db_helpers.rs, migrations, lib.rs 集成 |
| 2026-04-10 | Phase 2 完成 | 数据迁移：migrate_from_files 函数，从 JSON/JSONL 导入到 SQLite |
| 2026-04-10 | Phase 3 完成 | API 层：双写 activity log, SQLite 查询 thread/channel, metadata 更新 |
| 2026-04-10 | Phase 4 完成 | 文档更新：project-context.md v4, 移除 "不使用 SQLite" 规则 |
