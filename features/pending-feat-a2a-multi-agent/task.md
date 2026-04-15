# Tasks: feat-a2a-multi-agent

## Task Breakdown

### 1. Push Notification Receiver (`push.rs`)
- [ ] 内嵌 HTTP server 监听 push callback（复用 axum 或类似）
- [ ] POST /push handler 解析 A2A PushNotification 格式
- [ ] 事件类型分发（task_completed, task_failed, input_required 等）
- [ ] 通过 Tauri emit 发送前端事件
- [ ] PushNotificationConfig 管理（register / unregister / list configs）
- [ ] Webhook 签名验证（HMAC-SHA256，防伪造请求）

### 2. Task Delegation Engine (`delegation.rs`)
- [ ] `DelegateRequest` 结构体（from_agent, to_agent, task_description, context_summary, parent_task_id）
- [ ] `delegation::create()` — 创建委托，发送 A2A SendMessage 到目标 Agent
- [ ] `delegation::handle_result()` — 处理委托结果回传
- [ ] 上下文摘要生成（从当前 Thread/Channel 消息中提取关键信息）
- [ ] 父子 Task 关联存储
- [ ] 委托状态追踪（PENDING → SENT → ACKNOWLEDGED → IN_PROGRESS → COMPLETED/FAILED）

### 3. Cross-Agent Artifact Store (`artifact_store.rs`)
- [ ] `ArtifactRef` 结构体（id, producer_agent_id, name, file_path, content_hash, mime_type, created_at）
- [ ] `ArtifactStore` trait + 本地文件系统实现
- [ ] Artifact 注册（Agent 完成任务时自动注册产出物）
- [ ] Artifact 查询（按 agent_id / task_id / name 搜索）
- [ ] Artifact 内容获取（读文件或内存缓存）
- [ ] 消费记录追踪（consumer_agent_id + timestamp）

### 4. @mention 触发升级
- [ ] 扩展 mention.rs 解析器支持委托语义
- [ ] "@agent-name 请帮我..." → 识别为委托意图
- [ ] 从当前对话上下文自动提取摘要
- [ ] 选择同步（StreamMessage）或异步（Push）模式
- [ ] 与现有 feat-agent-a2a-trigger 逻辑整合

### 5. IPC Commands (`collaboration.rs`)
- [ ] `collaboration_delegate` — 发起委托
- [ ] `collaboration_list_delegations` — 列出活跃委托
- [ ] `collaboration_cancel_delegation` — 取消委托
- [ ] `collaboration_list_artifacts` — 列出可共享 Artifacts
- [ ] `collaboration_get_artifact` — 获取 Artifact 内容
- [ ] `collaboration_register_push_url` — 注册 push notification endpoint
- [ ] `collaboration_push_events` — 前端订阅 push 事件的 stream

### 6. 前端：协作 UI
- [ ] `CollaborationView.tsx` — Channel 内嵌的协作关系图/时间线
- [ ] `AgentTaskCard.tsx` — 单个 Agent 的任务状态卡片（含进度、Artifact 列表）
- [ ] 委托操作 UI（发起、取消、重试按钮）
- [ ] Push 通知 toast 提示组件
- [ ] Artifact 浏览器（按 Agent / Task 分组展示）

### 7. 前端：State & Hooks
- [ ] `useCollaboration` hook — 管理协作状态
- [ ] `usePushEvents` hook — 订阅 push 通知事件
- [ ] `useArtifacts` hook — Artifact 查询和管理
- [ ] Zustand store 扩展（collaboration slice）

### 8. 集成与测试
- [ ] 端到端委托流程测试（本地 Agent A → 本地 Agent B）
- [ ] Push Notification 回环测试（本地发 push → 接收 → UI 更新）
- [ ] Artifact 注册→查询→消费全链路测试
- [ ] 多 Agent 并发委托压力测试
- [ ] 与现有 Task 系统（feat-task-execution）的集成测试

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-14 | Feature created | Initial task breakdown |
