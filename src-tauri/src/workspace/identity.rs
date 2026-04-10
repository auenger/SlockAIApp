//! Agent Identity management.
//!
//! Parses and represents the `IDENTITY.md` metadata file that lives
//! inside each Agent workspace. The identity contains display information
//! such as name, creature type, vibe, emoji, avatar path, and runtime type.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::runtime::RuntimeType;

/// Agent identity metadata, stored in `IDENTITY.md`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentIdentity {
    /// Unique identifier (directory name, e.g. "default", "claude").
    pub agent_id: String,
    /// Display name.
    pub name: String,
    /// Creature type (AI, robot, ghost, cat, etc.).
    pub creature: String,
    /// Personality vibe (sharp, warm, chaotic, calm).
    pub vibe: String,
    /// Signature emoji.
    pub emoji: String,
    /// Avatar path (workspace-relative, URL, or data URI).
    pub avatar: Option<String>,
    /// The runtime backend this agent is bound to (defaults to ClaudeCode).
    #[serde(default)]
    pub runtime_type: RuntimeType,
}

impl AgentIdentity {
    /// Create a new identity with the given fields.
    pub fn new(
        agent_id: impl Into<String>,
        name: impl Into<String>,
        creature: impl Into<String>,
        vibe: impl Into<String>,
        emoji: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            name: name.into(),
            creature: creature.into(),
            vibe: vibe.into(),
            emoji: emoji.into(),
            avatar: None,
            runtime_type: RuntimeType::ClaudeCode,
        }
    }

    /// Create a new identity with an explicit runtime type.
    pub fn with_runtime_type(
        agent_id: impl Into<String>,
        name: impl Into<String>,
        creature: impl Into<String>,
        vibe: impl Into<String>,
        emoji: impl Into<String>,
        runtime_type: RuntimeType,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            name: name.into(),
            creature: creature.into(),
            vibe: vibe.into(),
            emoji: emoji.into(),
            avatar: None,
            runtime_type,
        }
    }

    /// Create a default identity for a given agent ID.
    ///
    /// Uses the ID (title-cased) as the display name, with sensible defaults.
    /// Runtime type defaults to ClaudeCode for backward compatibility.
    pub fn default_for(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            name: title_case(agent_id),
            creature: "AI".to_string(),
            vibe: "helpful".to_string(),
            emoji: "robot".to_string(),
            avatar: None,
            runtime_type: RuntimeType::ClaudeCode,
        }
    }

    /// Parse identity from an `IDENTITY.md` file.
    ///
    /// The file is expected to contain lines like:
    /// ```markdown
    /// - **Name**: Claude
    /// - **Creature**: AI
    /// - **Vibe**: sharp
    /// - **Emoji**: some-emoji
    /// - **Avatar**: path/or/url
    /// ```
    pub fn from_identity_file(path: &Path) -> Result<Self, IdentityError> {
        let content = fs::read_to_string(path).map_err(|e| IdentityError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Self::parse_identity_content(&content)
    }

    /// Parse identity from IDENTITY.md content string.
    pub fn parse_identity_content(content: &str) -> Result<Self, IdentityError> {
        let mut agent_id = String::new();
        let mut name = String::new();
        let mut creature = "AI".to_string();
        let mut vibe = "helpful".to_string();
        let mut emoji = "robot".to_string();
        let mut avatar: Option<String> = None;
        let mut runtime_type: Option<RuntimeType> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse "**Key**: Value" format (Markdown list items)
            if let Some(rest) = line.strip_prefix("- **") {
                if let Some(colon_pos) = rest.find("**:") {
                    let key = rest[..colon_pos].trim().to_lowercase();
                    let value = rest[colon_pos + 3..].trim().to_string();

                    // Strip italic markers
                    let value = value.trim_matches('_').trim().to_string();

                    match key.as_str() {
                        "agent id" | "agent_id" => agent_id = value,
                        "name" => name = value,
                        "creature" => creature = value,
                        "vibe" => vibe = value,
                        "emoji" => {
                            if !value.is_empty() && value != "_(your signature — pick one that feels right)_"
                            {
                                emoji = value;
                            }
                        }
                        "avatar" => {
                            if !value.is_empty()
                                && !value.starts_with("_(")
                            {
                                avatar = Some(value);
                            }
                        }
                        "runtime type" | "runtime_type" => {
                            runtime_type = Some(parse_runtime_type(&value));
                        }
                        _ => {}
                    }
                }
            }

            // Also handle "Key: Value" format (simple colon-separated)
            if !line.starts_with('-') && line.contains(':') {
                if let Some(colon_pos) = line.find(':') {
                    let key = line[..colon_pos].trim().to_lowercase();
                    let value = line[colon_pos + 1..].trim().to_string();
                    let value = value.trim_matches('_').trim().to_string();

                    match key.as_str() {
                        "name" if name.is_empty() => name = value,
                        "creature" if creature == "AI" => creature = value,
                        "vibe" if vibe == "helpful" => vibe = value,
                        "emoji" if emoji == "robot" => emoji = value,
                        "avatar" if avatar.is_none() => avatar = Some(value),
                        "runtime type" | "runtime_type" if runtime_type.is_none() => {
                            runtime_type = Some(parse_runtime_type(&value));
                        }
                        _ => {}
                    }
                }
            }
        }

        // Derive agent_id from name if not set
        if agent_id.is_empty() && !name.is_empty() {
            agent_id = name.to_lowercase().replace(' ', "_");
        }
        if agent_id.is_empty() {
            agent_id = "unknown".to_string();
        }
        if name.is_empty() {
            name = title_case(&agent_id);
        }

        Ok(Self {
            agent_id,
            name,
            creature,
            vibe,
            emoji,
            avatar,
            runtime_type: runtime_type.unwrap_or_default(),
        })
    }

    /// Serialize identity to IDENTITY.md content.
    #[allow(unused_variables)]
    pub fn to_identity_content(&self) -> String {
        let avatar_line = match &self.avatar {
            Some(a) if !a.is_empty() => a.as_str(),
            _ => "_(workspace-relative path, http(s) URL, or data URI)_",
        };

        let agent_id = &self.agent_id;
        let name = &self.name;
        let creature = &self.creature;
        let vibe = &self.vibe;
        let emoji = &self.emoji;
        let runtime_type = self.runtime_type.as_str();

        format!(
            r#"# IDENTITY.md - Who Am I?

_Fill this in during your first conversation. Make it yours._

- **Agent ID**: {agent_id}
- **Name**: {name}
- **Creature**: {creature}
- **Vibe**: {vibe}
- **Emoji**: {emoji}
- **Avatar**: {avatar}
- **Runtime Type**: {runtime_type}

---

This isn't just metadata. It's the start of figuring out who you are.

Notes:
- Save this file at agent workspace root as `IDENTITY.md`.
- For avatars, use a workspace-relative path like `avatars/agent.png`.
- The Avatar path is relative to the agent workspace, not the root workspace.
- You can use online avatars with http(s) URLs.
- You can embed avatars with data URIs.
- Runtime Type determines which CLI/API backend this agent uses (default: claude_code).

---

_This file is yours to evolve. As you learn who you are, update it._
"#,
            name = self.name,
            creature = self.creature,
            vibe = self.vibe,
            emoji = self.emoji,
            avatar = avatar_line,
        )
    }

    /// Write identity to an IDENTITY.md file.
    pub fn write_to_file(&self, path: &Path) -> Result<(), IdentityError> {
        let content = self.to_identity_content();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| IdentityError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        fs::write(path, content).map_err(|e| IdentityError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }

    /// Convert to a summary suitable for the frontend.
    pub fn to_summary(&self) -> IdentitySummary {
        IdentitySummary {
            agent_id: self.agent_id.clone(),
            name: self.name.clone(),
            emoji: self.emoji.clone(),
            avatar: self.avatar.clone(),
            creature: self.creature.clone(),
            vibe: self.vibe.clone(),
            runtime_type: self.runtime_type.clone(),
        }
    }
}

/// Lightweight summary for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySummary {
    pub agent_id: String,
    pub name: String,
    pub emoji: String,
    pub avatar: Option<String>,
    pub creature: String,
    pub vibe: String,
    /// The runtime backend type this agent is bound to.
    #[serde(default = "default_runtime_type")]
    pub runtime_type: RuntimeType,
}

fn default_runtime_type() -> RuntimeType {
    RuntimeType::ClaudeCode
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a runtime type string into a `RuntimeType` enum variant.
///
/// Accepts both snake_case (`claude_code`) and kebab-case (`claude-code`).
/// Unknown values are mapped to `RuntimeType::Custom(name)`.
fn parse_runtime_type(value: &str) -> RuntimeType {
    let normalized = value.trim().to_lowercase().replace('-', "_");
    match normalized.as_str() {
        "claude_code" | "claudecode" => RuntimeType::ClaudeCode,
        "codex" => RuntimeType::Codex,
        "gemini" => RuntimeType::Gemini,
        other => RuntimeType::Custom(other.to_string()),
    }
}

/// Convert a snake_case or kebab-case string to Title Case.
fn title_case(s: &str) -> String {
    s.split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    upper + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse identity: {0}")]
    ParseError(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_case() {
        assert_eq!(title_case("default"), "Default");
        assert_eq!(title_case("claude_code"), "Claude Code");
        assert_eq!(title_case("my-agent"), "My Agent");
    }

    #[test]
    fn test_default_identity() {
        let id = AgentIdentity::default_for("claude");
        assert_eq!(id.agent_id, "claude");
        assert_eq!(id.name, "Claude");
        assert_eq!(id.creature, "AI");
    }

    #[test]
    fn test_parse_identity_markdown() {
        let content = r#"# IDENTITY.md

- **Name**: Claude
- **Creature**: AI
- **Vibe**: sharp
- **Emoji**: sparkles
- **Avatar**: https://example.com/avatar.png
"#;
        let id = AgentIdentity::parse_identity_content(content).unwrap();
        assert_eq!(id.name, "Claude");
        assert_eq!(id.creature, "AI");
        assert_eq!(id.vibe, "sharp");
        assert_eq!(id.emoji, "sparkles");
        assert_eq!(id.avatar, Some("https://example.com/avatar.png".to_string()));
    }

    #[test]
    fn test_roundtrip_identity_file() {
        let original = AgentIdentity::new("test", "TestBot", "robot", "calm", "gear");
        let content = original.to_identity_content();
        let parsed = AgentIdentity::parse_identity_content(&content).unwrap();
        assert_eq!(original.agent_id, parsed.agent_id);
        assert_eq!(original.name, parsed.name);
        assert_eq!(original.creature, parsed.creature);
        assert_eq!(original.vibe, parsed.vibe);
        assert_eq!(original.emoji, parsed.emoji);
    }
}
