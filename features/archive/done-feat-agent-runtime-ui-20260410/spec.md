# Feature: feat-agent-runtime-ui Agent 创建 UI Runtime 选择

## Basic Information
- **ID**: feat-agent-runtime-ui
- **Name**: Agent 创建 UI Runtime 选择
- **Priority**: 75
- **Size**: S
- **Dependencies**: feat-agent-runtime-model
- **Parent**: feat-agent-runtime-select
- **Children**: (none)
- **Created**: 2026-04-10

## Description

在 Agent 创建/编辑 UI 中增加 runtime 类型选择器。用户创建 Agent 时可以选择 Claude Code、Codex 等不同的 agent client 作为该 Agent 的 runtime 后端。同时显示各 runtime 的安装状态和可用性。

## User Value Points

1. **Runtime 选择器 UI**: 用户在创建 Agent 时直观地选择 runtime 类型
2. **Runtime 状态可视化**: 实时显示各 runtime 的安装状态、版本号，未安装的显示安装提示

## Context Analysis

### Reference Code
- `src/components/CreateAgentModal.tsx` — 现有 Agent 创建 Modal（需扩展）
- `src/lib/ipc.ts` — IPC 调用层（需增加 runtime 相关 API）
- `src/types.ts` — 类型定义（需增加 RuntimeType、RuntimeInfo 类型）
- `src/hooks/useAgentStatus.ts` — Agent 状态 hook

### Related Features
- `feat-agent-runtime-model` (依赖) — 后端数据模型和 IPC commands
- `feat-agent-runtime-exec` (兄弟) — 对话执行层

## Technical Solution

### 1. 前端类型扩展

```typescript
// types.ts
type RuntimeType = "claude_code" | "codex" | "gemini" | "custom";

interface RuntimeInfo {
  runtime_type: RuntimeType;
  name: string;
  status: AgentRuntimeStatusType;
  version?: string;
  binary_path?: string;
  install_hint?: string;
  capabilities: AgentCapability[];
}

interface CreateAgentRequest {
  name: string;
  emoji: string;
  runtime_type: RuntimeType;  // 新增
  system_prompt?: string;      // 新增
}
```

### 2. CreateAgentModal 扩展

在现有 Modal 中增加 runtime 选择步骤：

```
┌─────────────────────────────────────┐
│  Create New Agent                    │
│                                      │
│  Name: [______________]              │
│  Emoji: [robot ▼]                    │
│                                      │
│  ── Runtime ──────────────────────── │
│  ○ Claude Code   ✓ v1.0.3 (可用)    │
│  ○ Codex         ✗ 未安装           │
│    > npm install -g @openai/codex   │
│  ○ Gemini        ✓ v0.2.0 (可用)    │
│                                      │
│  ── 高级设置 ──────────────────────  │
│  System Prompt (可选):               │
│  [________________________________]  │
│                                      │
│          [Cancel]  [Create Agent]     │
└─────────────────────────────────────┘
```

### 3. Runtime 状态检测 Hook

```typescript
// hooks/useRuntimeStatus.ts
function useRuntimeStatus() {
  const [runtimes, setRuntimes] = useState<RuntimeInfo[]>([]);

  useEffect(() => {
    invoke<RuntimeInfo[]>('list_runtimes').then(setRuntimes);
  }, []);

  return { runtimes, refresh: () => invoke('scan_agent_runtimes') };
}
```

### 4. IPC 层扩展

```typescript
// lib/ipc.ts
export const ipc = {
  agent: {
    // 现有
    create: (req: CreateAgentRequest) => invoke('create_agent', req),
    list: () => invoke('list_agents'),
    // 新增
    listRuntimes: () => invoke<RuntimeInfo[]>('list_runtimes'),
    getRuntimeInfo: (type: RuntimeType) => invoke('get_runtime_info', { runtimeType: type }),
    scanRuntimes: () => invoke('scan_agent_runtimes'),
  },
};
```

### 5. Agent Profile 页面增强

在 Agent 详情页增加 runtime 信息展示区域：
- 显示当前 runtime 类型和状态
- 提供切换 runtime 的选项（编辑模式）

## Acceptance Criteria (Gherkin)

### User Story
As a user, I want to select which AI runtime my agent uses so that I can leverage different AI tools for different tasks.

### Scenarios

```gherkin
Scenario: Create agent with Claude Code runtime
  Given the user opens the Create Agent modal
  And Claude Code is installed and available
  When the user selects "Claude Code" as runtime
  And fills in the agent name
  And clicks "Create Agent"
  Then the agent is created with runtime_type "claude_code"

Scenario: Show install hint for unavailable runtime
  Given Codex CLI is not installed
  When the user opens the Create Agent modal
  Then Codex option shows "not installed" with install command hint

Scenario: Default runtime is Claude Code
  Given the user opens the Create Agent modal
  Then "Claude Code" is pre-selected as the default runtime

Scenario: Runtime status auto-detects on modal open
  Given the Create Agent modal opens
  When the runtime detection completes
  Then each runtime shows its current availability status
```

### UI/Interaction Checkpoints
- [ ] Runtime 选择器使用 Radio Group 或 Card 选择样式
- [ ] 可用 runtime 显示绿色勾号 + 版本号
- [ ] 不可用 runtime 显示红色叉号 + 安装命令
- [ ] 默认选中 Claude Code
- [ ] Modal 打开时自动触发 runtime 检测

### General Checklist
- [ ] CreateAgentRequest 包含 runtime_type
- [ ] CreateAgentModal 增加步骤或分区显示 runtime 选择
- [ ] Runtime 状态实时检测
- [ ] Agent Profile 页面显示 runtime 信息

## Merge Record
- **Completed**: 2026-04-10T19:20:00+08:00
- **Branch**: feature/feat-agent-runtime-ui
- **Merge Commit**: 40f159a
- **Archive Tag**: feat-agent-runtime-ui-20260410
- **Conflicts**: none
- **Verification**: passed (4/4 Gherkin scenarios, 0 TS errors)
- **Stats**: started 2026-04-10T19:00:00+08:00, duration ~20min, 1 commit, 6 files changed
