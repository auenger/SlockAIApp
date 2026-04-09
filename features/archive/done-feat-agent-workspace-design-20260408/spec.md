# Feature: feat-agent-workspace-design Agent Workspace 与身份系统设计

## Basic Information
- **ID**: feat-agent-workspace-design
- **Name**: Agent Workspace 与身份系统设计
- **Priority**: 85
- **Size**: M
- **Dependencies**: feat-project-init
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-08

## Description
借鉴 anyclaw 项目的多 Agent Workspace 设计，为 SlockAI 设计完整的 Agent Workspace 隔离体系和身份/人格系统。包括：

1. **Agent Workspace 隔离** — 每个 Agent 拥有独立的文件目录，存储对话记录、上下文、输出和配置
2. **身份/人格系统** — 通过 SOUL.md、IDENTITY.md 等模板文件定义 Agent 的性格、行为规则和元信息
3. **模板系统** — 初始化 Workspace 时自动创建标准模板文件

## User Value Points

### VP1: Agent Workspace 物理隔离
每个 Agent 独立的工作区目录，互不干扰，支持安全的文件操作和上下文隔离。

### VP2: Agent 人格与身份定义
通过 Markdown 模板文件（SOUL.md/IDENTITY.md）为每个 Agent 定义独立的性格、行为准则和视觉标识，使不同 Agent 有差异化表现。

### VP3: 模板初始化与同步
首次创建 Agent Workspace 时自动生成标准模板文件，支持后续增量同步（不覆盖用户已修改的文件）。

## Context Analysis

### Reference Code
- **anyclaw 项目**: `reference/anyclaw/`
  - `anyclaw/agents/manager.py` — AgentManager 多 Agent 管理
  - `anyclaw/agents/identity.py` — IdentityManager 身份管理
  - `anyclaw/workspace/manager.py` — WorkspaceManager 工作区管理
  - `templates/` — 所有模板文件（SOUL.md, IDENTITY.md, USER.md, AGENTS.md, TOOLS.md, HEARTBEAT.md）

### Related Documents
- anyclaw 的 workspace 目录结构设计
- anyclaw 的模板文件设计哲学（SOUL.md 的"成为某个人"理念）

### Related Features
- feat-project-init（项目脚手架）

## Technical Solution

### 借鉴 anyclaw 的设计要点

#### 1. Workspace 目录结构（适配 SlockAI）

```
workspaces/
├── SOUL.md              # 全局 Agent 人格（默认）
├── USER.md              # 用户档案
├── AGENTS.md            # Agent 行为指令
├── TOOLS.md             # 工具使用说明
├── memory/              # 记忆存储
│   ├── MEMORY.md        # 长期记忆
│   └── HISTORY.md       # 历史摘要
└── agents/              # 多 Agent 目录
    ├── default/         # 默认 Agent
    │   ├── IDENTITY.md  # 身份元信息
    │   ├── SOUL.md      # 个性化人格（覆盖全局）
    │   ├── conversations/  # 对话记录 (JSONL)
    │   ├── context/        # 上下文快照
    │   ├── output/         # Agent 输出
    │   ├── skills/         # Agent 技能
    │   └── config/         # Agent 配置
    ├── claude/          # Claude Agent
    │   ├── IDENTITY.md
    │   ├── SOUL.md
    │   └── ...
    └── codex/           # Codex Agent
        ├── IDENTITY.md
        ├── SOUL.md
        └── ...
```

#### 2. 模板文件设计

##### SOUL.md — Agent 人格定义
定义 Agent 的核心性格、行为准则和价值观：
- **Core Truths**: 核心行为原则
- **Boundaries**: 行为边界
- **Vibe**: 沟通风格
- **Continuity**: 持久化策略

##### IDENTITY.md — Agent 身份元信息
轻量级元数据：
- Name、Creature（类型）、Vibe（风格）、Emoji、Avatar

##### USER.md — 用户档案
存储用户偏好：
- 基本信息、沟通风格、技术水平、工作背景

##### AGENTS.md — Agent 行为指令
Agent 的操作指南：
- 工具使用规则、技能调度、任务管理

#### 3. 与 SlockAI 核心概念的映射

| SlockAI 概念 | anyclaw 参考 | 适配说明 |
|-------------|-------------|---------|
| Workspace = Agent 文件目录 | agents/{name}/workspace/ | 每个 Agent 独立目录 |
| @Agent = 触发器 | AgentManager.switch_agent() | 切换时加载对应 workspace |
| 上下文编排引擎 | ContextBuilder | 加载 SOUL.md/IDENTITY.md 作为上下文前缀 |
| JSONL 驱动 | SessionManager | conversations/ 目录存储 JSONL |
| Agent 人格 | SOUL.md + IDENTITY.md | 模板文件定义人格 |

#### 4. Rust 实现方向

```rust
// Agent Workspace 管理
struct AgentWorkspace {
    base_path: PathBuf,
    conversations_dir: PathBuf,
    context_dir: PathBuf,
    output_dir: PathBuf,
}

// Agent 身份
struct AgentIdentity {
    name: String,
    creature: String,
    vibe: String,
    emoji: String,
    avatar: Option<String>,
}

// Agent 管理器
struct AgentManager {
    agents: HashMap<String, Agent>,
    active_agent: String,
    workspace_root: PathBuf,
}
```

<!-- 实现阶段补充 -->

## Acceptance Criteria (Gherkin)

### User Story
作为一个 SlockAI 用户，我希望每个 Agent 有独立的工作区和可定制的人格，这样不同 Agent 能在不同场景下以不同风格为我服务。

### Scenarios (Given/When/Then)

#### Scenario 1: Agent Workspace 初始化
```gherkin
Given SlockAI 首次启动
When 系统创建默认 Agent Workspace
Then workspaces/ 目录被创建
And 包含 SOUL.md, USER.md, AGENTS.md, TOOLS.md 模板文件
And 包含 agents/default/ 子目录
And agents/default/ 包含 IDENTITY.md 和 SOUL.md
And agents/default/ 包含 conversations/, context/, output/ 目录
```

#### Scenario 2: 多 Agent Workspace 隔离
```gherkin
Given 系统已有 default 和 claude 两个 Agent
When 用户在 Channel 中 @claude
Then 系统加载 claude Agent 的 workspace
And claude 的 SOUL.md 被用作上下文前缀
And claude 的对话记录写入 agents/claude/conversations/
And 不影响 default Agent 的数据
```

#### Scenario 3: SOUL.md 人格定制
```gherkin
Given 用户编辑了 agents/claude/SOUL.md
When @claude 被触发
Then Claude Agent 的回复风格符合 SOUL.md 中定义的 Vibe
And Core Truths 中的行为原则被遵守
And Boundaries 中的限制被尊重
```

#### Scenario 4: 模板同步不覆盖
```gherkin
Given 用户已自定义 agents/default/SOUL.md
When 系统执行模板同步
Then 已有的 SOUL.md 不被覆盖
And 缺失的模板文件被创建
```

### UI/Interaction Checkpoints
- Agent 选择器显示 Agent 的 Emoji 和名称（来自 IDENTITY.md）
- Agent 切换时加载对应的 Workspace 图标/颜色
- 设置页面允许编辑 SOUL.md 和 IDENTITY.md

### General Checklist
- [ ] Workspace 目录结构符合设计
- [ ] 模板文件内容完整且可定制
- [ ] 多 Agent 间数据完全隔离
- [ ] 上下文编排引擎正确加载 SOUL.md/IDENTITY.md
- [ ] 模板同步逻辑只创建缺失文件
