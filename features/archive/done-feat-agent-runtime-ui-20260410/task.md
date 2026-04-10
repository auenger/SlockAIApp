# Tasks: feat-agent-runtime-ui

## Task Breakdown

### 1. 前端类型定义
- [x] `types.ts` 增加 `RuntimeType` 类型 (already done by feat-agent-runtime-model)
- [x] `types.ts` 增加 `RuntimeInfo` interface (already done by feat-agent-runtime-model)
- [x] `types.ts` 更新 `CreateAgentRequest` 增加 `runtime_type` 字段 (already done by feat-agent-runtime-model)

### 2. IPC 层扩展
- [x] `ipc.ts` 增加 `listRuntimes()` 方法 (already done by feat-agent-runtime-model as listAgentRuntimes)
- [x] `ipc.ts` 增加 `getRuntimeInfo(type)` 方法 (already done by feat-agent-runtime-model)
- [x] `ipc.ts` 增加 `scanRuntimes()` 方法 (already done by feat-agent-runtime-model as scanAgentRuntimes)
- [x] `ipc.ts` 更新 `createAgent()` 参数 (already done by feat-agent-runtime-model)

### 3. Runtime 状态 Hook
- [x] 创建 `useRuntimeStatus.ts` hook
- [x] 实现自动检测 + 手动刷新
- [x] 返回 runtimes 列表和状态

### 4. CreateAgentModal 改造
- [x] Modal 打开时触发 runtime 检测
- [x] 增加 Runtime 选择区域（Radio Group / Card）
- [x] 可用 runtime 显示 ✓ + 版本号
- [x] 不可用 runtime 显示 ✗ + 安装提示
- [x] 默认选中 Claude Code
- [x] createAgent 调用传递 runtime_type

### 5. Agent Profile 页面增强
- [x] Agent 详情显示当前 runtime 类型
- [x] Agent 详情显示 runtime 状态
- [x] （可选）支持编辑时切换 runtime — deferred to future feature

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-10 | Feature created | 拆分自 feat-agent-runtime-select |
| 2026-04-10 | Implementation complete | Tasks 1-2 done by feat-agent-runtime-model. Tasks 3-5 implemented. Also fixed pre-existing mock data type errors. |
