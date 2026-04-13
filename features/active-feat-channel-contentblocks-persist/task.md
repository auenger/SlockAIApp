# Tasks: feat-channel-contentblocks-persist

## Task Breakdown

### 1. Rust 类型层
- [x] `channel.rs`: ChannelMessage 新增 `content_blocks: Option<Vec<serde_json::Value>>` 字段（含 serde 属性）

### 2. Rust 保存逻辑
- [x] `channel.rs execute_single_agent_inner`: 新增 `collected_blocks` 累积器
- [x] 事件循环中 assistant/user 事件带 content_blocks 时追加
- [x] ChannelMessage 构造时写入 `content_blocks`

### 3. Rust 事件传递
- [x] `channel-response` emit payload 增加 `content_blocks`

### 4. TypeScript 类型
- [x] `types.ts`: ChannelMessage 增加 `content_blocks?: ContentBlock[]`
- [x] `types.ts`: ChannelResponseEvent 增加 `content_blocks?: ContentBlock[]`

### 5. TypeScript Hook
- [x] `useChannel.ts`: channel-response handler 消息对象携带 content_blocks

### 6. UI 渲染
- [x] `MainContent.tsx`: channelDisplayMessages 保留 content_blocks
- [x] `MainContent.tsx`: 历史 agent 消息渲染 ContentBlockCard

### 7. 附带改动（已完成）
- [x] `zone_protocol.rs`: 移除规则 4（Agent 不再自报姓名）
- [x] `MainContent.tsx`: Streaming 指示器改为三跳动小点
- [x] `claude.rs`: system 事件文本优化（显示 "Session initialized · model"）
- [x] `claude.rs`: user 事件 content 解析（tool_result blocks）
- [x] `useChannel.ts`: counter-based 完成检测 + isSendingRef 并发保护

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-13 | 规则 4 移除 + streaming 指示器修改完成 | MainContent.tsx, zone_protocol.rs |
| 2026-04-13 | content_blocks 持久化完整实现 | Rust + TS + UI 全栈 |
| 2026-04-13 | Rust 编译通过 (`cargo check`) | OK |
| 2026-04-13 | 前端编译通过 (`tsc --noEmit` + `vite build`) | OK |
