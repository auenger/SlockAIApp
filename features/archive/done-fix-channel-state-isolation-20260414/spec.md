# Feature: fix-channel-state-isolation Channel 切换时 Agent Thinking 状态泄漏修复

## Basic Information
- **ID**: fix-channel-state-isolation
- **Name**: Channel 切换时 Agent Thinking/Streaming 状态隔离
- **Priority**: 80
- **Size**: S
- **Dependencies**: none
- **Parent**: null
- **Children**: none
- **Created**: 2026-04-14

## Description

在 Channel A 中 @agent 执行任务时，agent 进入 thinking 状态，此时切换到 Channel B，Channel B 也会显示 thinking 状态。每个 Channel 应该有独立的逻辑和对话状态。

**根因分析：** `useChannel.ts` 中的 `isStreaming`、`isThinking`、`streamingText`、`agentStreams` 是全局 React state，没有按 channel ID 隔离。切换 channel 时不清除上一个 channel 的 streaming 状态，导致状态泄漏到所有 channel 视图。

## User Value Points

1. **Channel 状态隔离** — 每个 Channel 有独立的 streaming/thinking 状态，切换 Channel 时不会看到其他 Channel 的 agent 运行状态

## Context Analysis

### Reference Code
- `src/lib/useChannel.ts` — 核心 hook，全局状态未隔离（L107-116）
- `src/App.tsx` — handleChannelSelect 不清除 streaming 状态（L112-118）
- `src/components/MainContent.tsx` — 消费 isStreaming/isThinking 状态渲染 UI

### Root Cause (3 层)

1. **全局状态变量** — `isStreaming`、`isThinking`、`streamingText`、`agentStreams` 是单例 state，非 per-channel
2. **切换不清除** — `selectChannel()` 不清理上一个 channel 的 streaming 状态
3. **事件监听不区分** — 虽然事件有 `channel_id` 过滤，但 state 更新是全局的

### Related Features
- `feat-channel-agent-thinking` (已完成) — 引入了 thinking 状态渲染
- `fix-channel-ui-bugs` (已完成) — 之前修复过 thinking 状态问题
- `fix-channel-msg-render` (已完成) — 修复过 Channel 消息即时渲染 & 状态清理

## Technical Solution

### 方案：Per-Channel State Map

将全局单例状态改为 `Map<channelId, ChannelStreamState>`：

```typescript
interface ChannelStreamState {
  isStreaming: boolean;
  isThinking: boolean;
  streamingText: string;
  agentStreams: AgentStreamState[];
}

// 替换原有的 useState
const [channelStreamStates, setChannelStreamStates] = useState<Map<string, ChannelStreamState>>(new Map());
```

**关键改动点：**

1. **`useChannel.ts`** — 将 4 个全局 state 合并为 per-channel Map
   - `selectChannel()` 切换时恢复目标 channel 的状态
   - 所有 state 更新操作改为 Map 操作（按 channelId 写入）
   - hook 返回的 isStreaming/isThinking 从当前 activeChannel 对应的 Map entry 读取

2. **`selectChannel()`** — 切换时加载目标 channel 的已有 streaming 状态（可能后台还在跑）

3. **Event listeners** — 已经有 `channel_id` 过滤，只需确保 state 写入 Map 正确 channel entry

### 影响范围
- `src/lib/useChannel.ts` — 主要改动
- `src/components/MainContent.tsx` — 可能需适配接口变化
- 其他消费 isStreaming/isThinking 的组件

## Acceptance Criteria (Gherkin)

### User Story
作为用户，我希望每个 Channel 的 Agent 运行状态完全独立，这样我可以在一个 Channel 等待 Agent 完成的同时，自由切换到其他 Channel 正常使用。

### Scenarios (Given/When/Then)

**Scenario 1: Channel A thinking 时不影响 Channel B**
```gherkin
Given Channel A 中 Agent 正在 thinking/streaming
And Channel B 没有 Agent 运行
When 用户切换到 Channel B
Then Channel B 不显示任何 thinking/streaming 状态
And Channel B 显示正常的空闲界面
```

**Scenario 2: 切回正在运行的 Channel 恢复状态**
```gherkin
Given Channel A 中 Agent 正在 thinking
When 用户切换到 Channel B
And 切回 Channel A
Then Channel A 仍然显示 thinking/streaming 状态
And Agent 继续正常工作
```

**Scenario 3: 多 Channel 同时运行独立**
```gherkin
Given Channel A 和 Channel C 各有 Agent 正在运行
When 用户从 Channel A 切换到 Channel B 再到 Channel C
Then Channel B 不显示 thinking
And Channel C 显示正确的 thinking/streaming 状态
```

**Scenario 4: Agent 完成后状态正确清理**
```gherkin
Given Channel A 中 Agent 刚完成运行
When 用户切换到 Channel B
And 切回 Channel A
Then Channel A 不再显示 thinking 状态
And 消息正常渲染
```

### General Checklist
- [x] 全局单例状态改为 per-channel Map
- [x] selectChannel 正确加载/恢复目标 channel 状态
- [x] 后台运行的 agent 事件正确写入对应 channel 的 Map entry
- [x] 切换时无闪烁或错误状态
- [x] 不影响现有消息渲染和发送逻辑

## Merge Record
- **Completed**: 2026-04-14T15:34:00+08:00
- **Branch**: feature/fix-channel-state-isolation
- **Merge Commit**: f75e522
- **Archive Tag**: fix-channel-state-isolation-20260414
- **Conflicts**: None
- **Verification**: passed (4/4 Gherkin scenarios)
- **Files Changed**: 1 (src/lib/useChannel.ts, +96/-40 lines)
- **Duration**: ~1h 24m
