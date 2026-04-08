# Feature: feat-rename-agentszone Rename to AgentsZone

## Basic Information
- **ID**: feat-rename-agentszone
- **Name**: Rename to AgentsZone
- **Priority**: 60
- **Size**: S
- **Dependencies**: None
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-08

## Description
将当前项目中所有关于 SlockAI / slockai / slock 的 title、name、branding 全部替换为 AgentsZone / agentszone。涉及 HTML 页面标题、窗口标题、应用名、包名、产品标识符、Rust crate 名、欢迎消息、密钥环服务名、配置文件项目名、文档标题等。

## User Value Points
1. **品牌统一** - 所有用户可见的标题和名称统一为 AgentsZone，确保品牌一致性

## Context Analysis

### 需要修改的文件清单

#### 核心应用配置
| 文件 | 当前值 | 目标值 |
|------|--------|--------|
| `index.html:6` | `<title>SlockAI</title>` | `<title>AgentsZone</title>` |
| `package.json:2` | `"name": "slockai"` | `"name": "agentszone"` |
| `src-tauri/tauri.conf.json:3` | `"productName": "SlockAI"` | `"productName": "AgentsZone"` |
| `src-tauri/tauri.conf.json:5` | `"identifier": "com.slockai.app"` | `"identifier": "com.agentszone.app"` |
| `src-tauri/tauri.conf.json:15` | `"title": "SlockAI"` | `"title": "AgentsZone"` |
| `src-tauri/Cargo.toml:2` | `name = "slockai"` | `name = "agentszone"` |
| `src-tauri/Cargo.toml:4` | `description = "SlockAI - ..."` | `description = "AgentsZone - ..."` |
| `src-tauri/Cargo.toml:5` | `authors = ["SlockAI"]` | `authors = ["AgentsZone"]` |
| `src-tauri/Cargo.toml:14` | `name = "slockai_lib"` | `name = "agentszone_lib"` |

#### Rust 源码
| 文件 | 修改内容 |
|------|----------|
| `src-tauri/src/main.rs:5` | `slockai_lib::run()` → `agentszone_lib::run()` |
| `src-tauri/src/commands/mod.rs:268` | 欢迎消息中的 "SlockAI" → "AgentsZone" |
| `src-tauri/src/storage/keyring.rs:8` | `SERVICE_NAME = "SlockAI"` → `"AgentsZone"` |
| `src-tauri/src/workspace/templates.rs` | 模板注释和字符串中的 "SlockAI" → "AgentsZone" |

#### TypeScript 源码
| 文件 | 修改内容 |
|------|----------|
| `src/lib/useAgentRuntimes.ts:2` | JSDoc 注释中的 "SlockAI" → "AgentsZone" |

#### 配置文件
| 文件 | 修改内容 |
|------|----------|
| `feature-workflow/config.yaml:3` | `name: SlockAI` → `name: AgentsZone` |
| `feature-workflow/config.yaml:19,50` | `worktree_prefix: SlockAI` → `worktree_prefix: AgentsZone` |

#### 文档
| 文件 | 修改内容 |
|------|----------|
| `README.md` | 标题和描述中的 "SlockAI" → "AgentsZone" |
| `project-context.md` | 项目名引用 |

### 不修改的文件
- `features/archive/` - 已完成的历史记录，保持原样
- `ReactDemo/slockai-prototype/` - 旧原型目录，可后续清理
- `PMFile/`, `PMDM/` - 分析文档，属于历史参考资料
- `.github/workflows/` - CI 路径引用，待原型目录重命名时一并修改
- `PROJECT-REVIEW.md` - 项目 review 报告，历史记录
- `src/components/MainContent.tsx` 中的 `~/.slock/agents/` 路径 - 运行时路径，改动需谨慎评估

### Reference Code
- `package.json` - npm 包名
- `src-tauri/Cargo.toml` - Rust crate 配置
- `src-tauri/tauri.conf.json` - Tauri 应用配置

### Related Documents
- README.md
- project-context.md

### Related Features
- 无

## Technical Solution
使用全局搜索替换，逐文件修改所有 SlockAI/slockai/Slock 引用为 AgentsZone/agentszone。注意保持大小写一致性：
- `SlockAI` → `AgentsZone` (PascalCase, 用于标题/产品名)
- `slockai` → `agentszone` (lowercase, 用于包名/标识符)
- `slockai_lib` → `agentszone_lib` (crate lib 名)

## Acceptance Criteria (Gherkin)

### User Story
作为一个项目维护者，我希望项目所有用户可见的标题和名称都显示为 AgentsZone，以便建立统一的品牌形象。

### Scenarios (Given/When/Then)

```gherkin
Scenario: HTML 页面标题正确显示
  Given 项目已构建
  When 用户在浏览器中打开应用
  Then 页面标题应显示为 "AgentsZone"

Scenario: Tauri 窗口标题正确显示
  Given 应用已启动
  When Tauri 窗口加载完成
  Then 窗口标题栏应显示 "AgentsZone"

Scenario: Cargo 包名正确配置
  Given src-tauri/Cargo.toml 已更新
  When 执行 cargo build
  Then 构建应成功且 crate 名为 "agentszone"

Scenario: 应用标识符正确更新
  Given tauri.conf.json 已更新
  When 检查应用标识符
  Then identifier 应为 "com.agentszone.app"
```

### General Checklist
- [ ] 所有 title 标签已更新
- [ ] 所有 productName 已更新
- [ ] 所有 identifier 已更新
- [ ] Cargo crate 名和 lib 名已更新
- [ ] Rust 代码中的引用已同步更新
- [ ] 项目可正常构建和运行
