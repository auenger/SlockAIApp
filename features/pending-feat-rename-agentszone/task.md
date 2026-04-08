# Tasks: feat-rename-agentszone

## Task Breakdown

### 1. 核心配置文件
- [ ] 修改 `index.html` - 更新 `<title>` 标签
- [ ] 修改 `package.json` - 更新 name 字段
- [ ] 修改 `src-tauri/tauri.conf.json` - 更新 productName、identifier、title
- [ ] 修改 `src-tauri/Cargo.toml` - 更新 name、description、authors、lib name

### 2. Rust 源码
- [ ] 修改 `src-tauri/src/main.rs` - 更新 lib 引用
- [ ] 修改 `src-tauri/src/commands/mod.rs` - 更新欢迎消息
- [ ] 修改 `src-tauri/src/storage/keyring.rs` - 更新服务名
- [ ] 修改 `src-tauri/src/workspace/templates.rs` - 更新模板中的品牌引用

### 3. TypeScript 源码
- [ ] 修改 `src/lib/useAgentRuntimes.ts` - 更新 JSDoc 注释

### 4. 项目配置与文档
- [ ] 修改 `feature-workflow/config.yaml` - 更新 project name 和 worktree_prefix
- [ ] 修改 `README.md` - 更新标题和描述
- [ ] 修改 `project-context.md` - 更新项目名引用

### 5. 验证
- [ ] 执行 `cargo build` 验证 Rust 编译通过
- [ ] 执行 `npm install` / 构建验证前端无报错

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-08 | Feature created | 待开始实现 |
