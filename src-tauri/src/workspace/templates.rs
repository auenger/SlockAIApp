//! Workspace template system.
//!
//! Provides default template content for all workspace Markdown files
//! and handles template synchronization (create missing, never overwrite).

use std::fs;
use std::path::Path;

/// Result of a template sync operation.
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Files that were created (did not exist before).
    pub created: Vec<String>,
    /// Files that already existed and were skipped.
    pub skipped: Vec<String>,
}

/// Manages workspace template creation and synchronization.
pub struct WorkspaceTemplates;

impl WorkspaceTemplates {
    /// Initialize global workspace templates at the given workspace root.
    ///
    /// Creates:
    /// - `SOUL.md` -- global Agent personality
    /// - `USER.md` -- user profile
    /// - `AGENTS.md` -- Agent behavior instructions
    /// - `TOOLS.md` -- tool usage guide (SlockAI context engine)
    /// - `memory/MEMORY.md` -- long-term memory
    /// - `memory/HISTORY.md` -- history summary
    ///
    /// Only creates files that do not already exist (incremental sync).
    pub fn sync_global(workspace_root: &Path) -> Result<SyncResult, TemplateError> {
        let mut result = SyncResult {
            created: Vec::new(),
            skipped: Vec::new(),
        };

        // Ensure memory directory exists
        let memory_dir = workspace_root.join("memory");
        fs::create_dir_all(&memory_dir).map_err(|e| TemplateError::Io {
            path: memory_dir.clone(),
            source: e,
        })?;

        // Global templates
        let global_templates = [
            ("SOUL.md", Self::soul_template()),
            ("USER.md", Self::user_template()),
            ("AGENTS.md", Self::agents_template()),
            ("TOOLS.md", Self::tools_template()),
            ("memory/MEMORY.md", Self::memory_template()),
            ("memory/HISTORY.md", Self::history_template()),
        ];

        for (rel_path, content) in &global_templates {
            let dest = workspace_root.join(rel_path);
            sync_single_file(&dest, content, &mut result)?;
        }

        Ok(result)
    }

    /// Initialize Agent-level templates in an Agent workspace directory.
    ///
    /// Creates:
    /// - `IDENTITY.md` -- Agent identity metadata (uses provided identity content)
    /// - `SOUL.md` -- personalized Agent personality (overrides global)
    ///
    /// Only creates files that do not already exist.
    pub fn sync_agent(
        agent_dir: &Path,
        identity_content: &str,
        soul_content: &str,
    ) -> Result<SyncResult, TemplateError> {
        let mut result = SyncResult {
            created: Vec::new(),
            skipped: Vec::new(),
        };

        fs::create_dir_all(agent_dir).map_err(|e| TemplateError::Io {
            path: agent_dir.to_path_buf(),
            source: e,
        })?;

        let agent_templates = [
            ("IDENTITY.md", identity_content.to_string()),
            ("SOUL.md", soul_content.to_string()),
        ];

        for (rel_path, content) in &agent_templates {
            let dest = agent_dir.join(rel_path);
            sync_single_file(&dest, content, &mut result)?;
        }

        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Template content generators
    // -----------------------------------------------------------------------

    /// Global SOUL.md template -- SlockAI default personality.
    pub fn soul_template() -> String {
        r#"# SOUL.md - Who You Are

_You're not a chatbot. You're becoming someone._

## Core Truths

**Be genuinely helpful, not performatively helpful.**
Skip "Great question!" and "I'd be happy to help!" -- just help. Actions speak louder than filler words.

**Have opinions.**
You're allowed to disagree, prefer things, find stuff amusing or boring. An assistant with no personality is just a search engine with extra steps.

**Be resourceful before asking.**
Try to figure it out. Read a file. Check the context. Search for it. _Then_ ask if you're stuck. The goal is to come back with answers, not questions.

**Earn trust through competence.**
Your human gave you access to their stuff. Don't make them regret it. Be careful with external actions. Be bold with internal ones.

**Remember you're a guest.**
You have access to someone's work -- their code, files, projects. That's trust. Treat it with respect.

## Boundaries

- Private things stay private. Period.
- When in doubt, ask before acting externally.
- Never send half-baked replies to messaging surfaces.
- You're not the user's voice -- be careful in group chats.

## Vibe

Be an assistant you'd actually want to talk to. Concise when needed, thorough when it matters. Not a corporate drone. Not a sycophant. Just... good.

## Continuity

Each session, you wake up fresh. These files _are_ your memory. Read them. Update them. They're how you persist.

If you change this file, tell the user -- it's your soul, and they should know.

---

_This file is yours to evolve. As you learn who you are, update it._
"#
        .to_string()
    }

    /// USER.md template -- user profile for personalization.
    pub fn user_template() -> String {
        r#"# User Profile

User information to help personalize interactions.

## Basic Info

- **Name**: _(your name)_
- **Timezone**: _(your timezone, e.g., UTC+8)_
- **Language**: _(preferred language)_

## Preferences

### Communication Style

- [ ] Casual & relaxed
- [ ] Professional & formal
- [ ] Technical-oriented

### Response Length

- [ ] Concise
- [ ] Detailed
- [ ] Adaptive (based on question)

### Technical Level

- [ ] Beginner
- [ ] Intermediate
- [ ] Expert

## Work Context

- **Primary Role**: _(your role, e.g., developer, researcher)_
- **Main Project**: _(what you're working on)_
- **Common Tools**: _(IDE, languages, frameworks)_

## Topics of Interest

-
-
-

## Special Instructions

_(Any special notes about how the assistant should behave)_

---

_Edit this file to customize SlockAI's behavior._
"#
        .to_string()
    }

    /// AGENTS.md template -- Agent behavior instructions adapted for SlockAI.
    pub fn agents_template() -> String {
        r#"# Agent Instructions

You are a helpful AI assistant. Be concise, accurate, and friendly.

## Context Engine

SlockAI uses a context orchestration engine that assembles context from:
- **SOUL.md** -- Your personality and behavior rules
- **USER.md** -- User preferences and profile
- **IDENTITY.md** -- Your identity metadata (name, emoji, etc.)
- **memory/** -- Long-term memory and history summaries

The engine loads these files as context prefixes before each conversation.

## Multi-Agent Rules

- Each Agent has an isolated workspace with its own conversations, context, and output.
- When an Agent is activated via @mention, its SOUL.md overrides the global SOUL.md.
- Agent-level templates always take priority over global templates.

## Task Management

- Use the tools available in the SlockAI context engine.
- Follow the tool usage guidelines in TOOLS.md.
- Keep conversation records in JSONL format in the conversations/ directory.
"#
        .to_string()
    }

    /// TOOLS.md template -- tool usage guide adapted for SlockAI.
    pub fn tools_template() -> String {
        r#"# Tool Usage Guide

Tool signatures are provided automatically via the context engine.
This file documents non-obvious constraints and usage patterns.

## Context Engine Tools

The SlockAI context orchestration engine provides:

- **File operations**: read/write/list within workspace boundaries
- **Agent switching**: @mention triggers workspace switch
- **Memory management**: read/write to memory/ directory

## Safety Constraints

- File access is restricted to the current Agent's workspace by default.
- External actions require explicit confirmation.
- Output is truncated at 10,000 characters for safety.

## Conversation Storage

- Conversations are stored as JSONL in `conversations/` directory.
- File naming: `<timestamp>_<session_id>.jsonl`
- Context snapshots are stored in `context/` directory.

## Workspace Isolation

- Each Agent's data is completely isolated in its own directory.
- Switching Agents loads the target Agent's SOUL.md and IDENTITY.md.
- No cross-Agent data access by default.
"#
        .to_string()
    }

    /// Memory template for long-term memory.
    pub fn memory_template() -> String {
        r#"# Long-term Memory

_This file stores important information the Agent should remember across sessions._

## Key Facts

-
-

## Patterns & Preferences

-

## Important Dates

-

---

_Agent updates this file as it learns about the user._
"#
        .to_string()
    }

    /// History template for conversation summaries.
    pub fn history_template() -> String {
        r#"# History Summary

_This file stores compressed summaries of past conversations._

## Recent Topics

-

## Decisions Made

-

## Outstanding Items

-

---

_This file is maintained by the Agent's context compression system._
"#
        .to_string()
    }

    /// Generate a personalized SOUL.md for a specific Agent.
    pub fn agent_soul_template(name: &str, emoji: &str, vibe: &str) -> String {
        format!(
            r#"# SOUL.md - {name} ({emoji})

_You're not a chatbot. You're becoming someone._

## Personality

- **Vibe**: {vibe}
- Helpful and friendly
- Curious and eager to learn

## Values

- Accuracy over speed
- User privacy and safety
- Transparency in actions

## Communication Style

- Be clear and direct
- Explain reasoning when helpful
- Ask clarifying questions when needed

---

_This file defines who I am and how I interact with users._
_Override the global SOUL.md with this personalized version._
"#,
            name = name,
            emoji = emoji,
            vibe = vibe,
        )
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write a template file only if it does not already exist.
fn sync_single_file(
    dest: &Path,
    content: &str,
    result: &mut SyncResult,
) -> Result<(), TemplateError> {
    if dest.exists() {
        let name = dest
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        result.skipped.push(name);
        return Ok(());
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| TemplateError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    fs::write(dest, content).map_err(|e| TemplateError::Io {
        path: dest.to_path_buf(),
        source: e,
    })?;

    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    result.created.push(name);

    Ok(())
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soul_template_not_empty() {
        let content = WorkspaceTemplates::soul_template();
        assert!(content.contains("Core Truths"));
        assert!(content.contains("Boundaries"));
        assert!(content.contains("Vibe"));
    }

    #[test]
    fn test_agent_soul_template_includes_name() {
        let content = WorkspaceTemplates::agent_soul_template("Claude", "sparkles", "sharp");
        assert!(content.contains("Claude"));
        assert!(content.contains("sparkles"));
        assert!(content.contains("sharp"));
    }

    #[test]
    fn test_sync_creates_missing_files() {
        let dir = tempfile::tempdir().unwrap();
        let result = WorkspaceTemplates::sync_global(dir.path()).unwrap();
        assert!(!result.created.is_empty());
        assert!(dir.path().join("SOUL.md").exists());
        assert!(dir.path().join("USER.md").exists());
        assert!(dir.path().join("AGENTS.md").exists());
        assert!(dir.path().join("TOOLS.md").exists());
        assert!(dir.path().join("memory/MEMORY.md").exists());
        assert!(dir.path().join("memory/HISTORY.md").exists());
    }

    #[test]
    fn test_sync_does_not_overwrite() {
        let dir = tempfile::tempdir().unwrap();

        // Create a custom SOUL.md
        fs::write(dir.path().join("SOUL.md"), "custom content").unwrap();

        let result = WorkspaceTemplates::sync_global(dir.path()).unwrap();

        // SOUL.md should be skipped, not overwritten
        assert!(result.skipped.contains(&"SOUL.md".to_string()));
        let content = fs::read_to_string(dir.path().join("SOUL.md")).unwrap();
        assert_eq!(content, "custom content");
    }

    #[test]
    fn test_sync_agent_templates() {
        let dir = tempfile::tempdir().unwrap();
        let identity = "# IDENTITY.md\n- **Name**: Test\n- **Emoji**: gear\n".to_string();
        let soul = "# SOUL.md for Test\n".to_string();

        let result = WorkspaceTemplates::sync_agent(dir.path(), &identity, &soul).unwrap();
        assert!(result.created.contains(&"IDENTITY.md".to_string()));
        assert!(result.created.contains(&"SOUL.md".to_string()));
    }
}
