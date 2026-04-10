# Feature: feat-apikey-management-ui API Key 管理 UI

## Basic Information
- **ID**: feat-apikey-management-ui
- **Name**: API Key 管理 UI（低优先级，Runtime CLI 自管理认证）
- **Priority**: 10
- **Size**: S
- **Dependencies**: None
- **Parent**: null
- **Children**: []
- **Created**: 2026-04-10

## Description
实现 API Key 的可视化管理界面。当前 API Key 通过 keyring 存储（参见 `src-tauri/src/storage/keyring.rs`），本 feature 将提供一个前端界面来查看、添加和删除已配置的 API Key。

注意：优先级较低，因为 Runtime CLI 目前可以自行管理认证。

## User Value Points

### VP1: API Key 可视化管理
用户可以在界面中查看已配置的 API Key 列表（脱敏显示），添加新 Key 或删除已有 Key。

### VP2: Key 状态反馈
用户可以看到哪些 Key 有效、哪些已过期或即将过期。

## Context Analysis

### 前端相关文件
| 文件 | 说明 |
|------|------|
| `src/components/Sidebar.tsx` | 可能需要设置入口 |
| `src/components/MainContent.tsx` | 承载设置视图 |
| `src/lib/ipc.ts` | IPC 命令层 |

### 后端相关文件
| 文件 | 说明 |
|------|------|
| `src-tauri/src/storage/keyring.rs` | 现有 keyring 存储 |
| `src-tauri/src/commands/mod.rs` | Tauri 命令注册 |

### Related Documents
- project-context.md

### Related Features
- 无

## Technical Solution
待实现时细化。初步方向：
1. 复用现有 keyring 存储机制
2. 新增 API Key 的 list/add/delete commands（注意 list 时脱敏）
3. 前端新增设置页面或 Modal，承载 Key 管理功能

## Acceptance Criteria (Gherkin)

### User Story
作为一个用户，我希望在界面中管理 API Key，以便方便地配置和更新认证信息。

### Scenarios (Given/When/Then)

```gherkin
Scenario: 查看已存储的 API Key
  Given 系统中已存储了 API Key
  When 用户打开 API Key 管理页面
  Then 应显示 Key 列表（脱敏显示，如 sk-***...xyz）

Scenario: 添加新 API Key
  Given 用户在 API Key 管理页面
  When 用户输入新的 API Key 并保存
  Then Key 应被安全存储到 keyring 中

Scenario: 删除 API Key
  Given Key 列表中有至少一个 Key
  When 用户点击删除并确认
  Then 该 Key 应从 keyring 中移除
```

### General Checklist
- [x] API Key list/add/delete commands 实现
- [x] Key 脱敏显示
- [x] 前端管理 UI 完成
- [x] Key 安全存储（使用 keyring）

## Merge Record

- **Completed**: 2026-04-10T10:50:00+08:00
- **Merged Branch**: feature/feat-apikey-management-ui
- **Merge Commit**: 1b1330a
- **Archive Tag**: feat-apikey-management-ui-20260410
- **Conflicts**: None
- **Verification**: All 3/3 Gherkin scenarios passed. TypeScript + Rust compilation clean.
- **Stats**: 11 tasks, 11 files changed, 811 insertions, 1 commit
