# Tasks: feat-channel-infra

## Task Breakdown

### 1. Rust Backend - Channel 数据模型
- [ ] 定义 `Channel` struct（id, name, member_agent_ids, created_at, updated_at）
- [ ] 定义 `ChannelMember` struct（channel_id, agent_id, joined_at, role）
- [ ] 定义 `ChannelMessage` struct（id, channel_id, sender_type, sender_id, content, timestamp）
- [ ] Channel 存储位置：workspace 级别（非 agent 级别）

### 2. Rust Backend - Channel CRUD Commands
- [ ] `create_channel` — 创建 Channel，指定名称和 Agent 成员
- [ ] `list_channels` — 列出所有 Channel
- [ ] `get_channel` — 获取 Channel 详情（含成员列表）
- [ ] `update_channel` — 更新 Channel 名称/设置
- [ ] `delete_channel` — 删除 Channel 及其消息
- [ ] `add_channel_member` — 添加 Agent 到 Channel
- [ ] `remove_channel_member` — 从 Channel 移除 Agent

### 3. Rust Backend - Channel 消息
- [ ] `send_channel_message` — 在 Channel 中发送消息，路由到默认 Agent
- [ ] Channel 消息存储（JSONL 格式，复用 conversation-store 模式）

### 4. Frontend - Channel Types & IPC
- [ ] 扩展 `src/types.ts` — Channel（增加 members）, ChannelMessage 类型
- [ ] 扩展 `src/lib/ipc.ts` — Channel CRUD commands
- [ ] 新增 `useChannel` hook

### 5. Frontend - Sidebar Channel 改造
- [ ] 替换 hardcoded channel 列表为动态数据
- [ ] Channel 创建入口（+ 按钮创建弹窗）
- [ ] Channel 选中 → 传递到 MainContent

### 6. Frontend - Channel Chat UI
- [ ] MainContent 支持 Channel 视图（复用 Chat tab 模式）
- [ ] Channel header 显示成员 Agent 头像
- [ ] 消息输入框 placeholder 为 "Message #{channelName}"

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
