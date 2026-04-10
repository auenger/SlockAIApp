//! Skill management for Agent workspaces.
//!
//! Each Agent can have multiple Skills configured. Skills are stored as a
//! single JSON file in the agent's `skills/` directory.
//!
//! File path pattern:
//!   `agents/{agent_id}/skills/skills.json`
//!
//! Each Skill has a type (e.g. "mcp_server", "tool", "custom_command"),
//! a configuration object, and a status.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Skill data model
// ---------------------------------------------------------------------------

/// Skill type classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SkillType {
    /// MCP server connection
    McpServer,
    /// Tool invocation capability
    Tool,
    /// Custom command / prompt extension
    CustomCommand,
}

/// Skill running status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SkillStatus {
    /// Skill is active and operational
    Active,
    /// Skill is inactive / disabled
    Inactive,
    /// Skill encountered an error
    Error,
    /// Skill is connecting / initializing
    Connecting,
}

impl std::fmt::Display for SkillType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillType::McpServer => write!(f, "MCP Server"),
            SkillType::Tool => write!(f, "Tool"),
            SkillType::CustomCommand => write!(f, "Custom Command"),
        }
    }
}

impl std::fmt::Display for SkillStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillStatus::Active => write!(f, "Active"),
            SkillStatus::Inactive => write!(f, "Inactive"),
            SkillStatus::Error => write!(f, "Error"),
            SkillStatus::Connecting => write!(f, "Connecting"),
        }
    }
}

/// A single Skill configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique skill identifier.
    pub id: String,
    /// Agent this skill belongs to.
    pub agent_id: String,
    /// Human-readable skill name.
    pub name: String,
    /// Skill type.
    pub skill_type: SkillType,
    /// Skill configuration (flexible JSON object).
    pub config: serde_json::Value,
    /// Running status.
    pub status: SkillStatus,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
    /// Last updated timestamp (ISO 8601).
    pub updated_at: String,
}

/// The full skills file content stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillsFile {
    /// List of skills.
    pub skills: Vec<Skill>,
}

// ---------------------------------------------------------------------------
// Skills store
// ---------------------------------------------------------------------------

/// Manages reading and writing the skills JSON file for an agent.
pub struct SkillStore {
    /// Path to the agent's skills directory.
    skills_dir: PathBuf,
}

impl SkillStore {
    /// Create a new SkillStore for the given agent skills directory.
    pub fn new(skills_dir: impl Into<PathBuf>) -> Self {
        Self {
            skills_dir: skills_dir.into(),
        }
    }

    /// Get the path to the skills.json file.
    fn skills_file(&self) -> PathBuf {
        self.skills_dir.join("skills.json")
    }

    /// Ensure the skills directory exists.
    fn ensure_dir(&self) -> Result<(), SkillError> {
        fs::create_dir_all(&self.skills_dir).map_err(|e| SkillError::Io {
            path: self.skills_dir.clone(),
            source: e,
        })
    }

    /// Load all skills from disk. Returns an empty list if the file doesn't exist.
    pub fn load_all(&self) -> Result<Vec<Skill>, SkillError> {
        let path = self.skills_file();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&path).map_err(|e| SkillError::Io {
            path: path.clone(),
            source: e,
        })?;
        let file: SkillsFile = serde_json::from_str(&content).map_err(|e| SkillError::Serialization {
            message: e.to_string(),
        })?;
        Ok(file.skills)
    }

    /// Save all skills to disk.
    fn save_all(&self, skills: &[Skill]) -> Result<(), SkillError> {
        self.ensure_dir()?;
        let path = self.skills_file();
        let file = SkillsFile {
            skills: skills.to_vec(),
        };
        let content = serde_json::to_string_pretty(&file).map_err(|e| SkillError::Serialization {
            message: e.to_string(),
        })?;
        fs::write(&path, content).map_err(|e| SkillError::Io {
            path: path.clone(),
            source: e,
        })
    }

    /// Add a new skill.
    pub fn add(&self, skill: Skill) -> Result<Skill, SkillError> {
        let mut skills = self.load_all()?;
        // Check for duplicate ID
        if skills.iter().any(|s| s.id == skill.id) {
            return Err(SkillError::AlreadyExists { skill_id: skill.id });
        }
        skills.push(skill.clone());
        self.save_all(&skills)?;
        Ok(skill)
    }

    /// Update an existing skill by ID.
    pub fn update(&self, skill_id: &str, name: Option<String>, skill_type: Option<SkillType>, config: Option<serde_json::Value>, status: Option<SkillStatus>) -> Result<Skill, SkillError> {
        let mut skills = self.load_all()?;
        let skill = skills.iter_mut().find(|s| s.id == skill_id)
            .ok_or_else(|| SkillError::NotFound { skill_id: skill_id.to_string() })?;

        if let Some(n) = name { skill.name = n; }
        if let Some(t) = skill_type { skill.skill_type = t; }
        if let Some(c) = config { skill.config = c; }
        if let Some(s) = status { skill.status = s; }
        skill.updated_at = format_timestamp_iso();

        let updated = skill.clone();
        self.save_all(&skills)?;
        Ok(updated)
    }

    /// Delete a skill by ID.
    pub fn delete(&self, skill_id: &str) -> Result<(), SkillError> {
        let mut skills = self.load_all()?;
        let original_len = skills.len();
        skills.retain(|s| s.id != skill_id);
        if skills.len() == original_len {
            return Err(SkillError::NotFound { skill_id: skill_id.to_string() });
        }
        self.save_all(&skills)
    }

    /// Get a single skill by ID.
    pub fn get(&self, skill_id: &str) -> Result<Option<Skill>, SkillError> {
        let skills = self.load_all()?;
        Ok(skills.into_iter().find(|s| s.id == skill_id))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate a compact ISO 8601 timestamp.
fn format_timestamp_iso() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds,
    )
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [u64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Generate a random-ish skill ID.
pub fn generate_skill_id() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("skill_{:x}", now)
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("serialization error: {message}")]
    Serialization { message: String },

    #[error("skill not found: {skill_id}")]
    NotFound { skill_id: String },

    #[error("skill already exists: {skill_id}")]
    AlreadyExists { skill_id: String },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(id: &str, agent_id: &str, name: &str) -> Skill {
        Skill {
            id: id.to_string(),
            agent_id: agent_id.to_string(),
            name: name.to_string(),
            skill_type: SkillType::Tool,
            config: serde_json::json!({}),
            status: SkillStatus::Active,
            created_at: "2026-04-10T12:00:00Z".to_string(),
            updated_at: "2026-04-10T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_add_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let store = SkillStore::new(dir.path().join("skills"));

        let skill = make_skill("s1", "default", "Web Search");
        store.add(skill).unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "s1");
        assert_eq!(loaded[0].name, "Web Search");
    }

    #[test]
    fn test_load_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = SkillStore::new(dir.path().join("skills"));

        let loaded = store.load_all().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_update() {
        let dir = tempfile::tempdir().unwrap();
        let store = SkillStore::new(dir.path().join("skills"));

        store.add(make_skill("s1", "default", "Original")).unwrap();
        let updated = store.update("s1", Some("Updated".to_string()), None, None, None).unwrap();
        assert_eq!(updated.name, "Updated");

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded[0].name, "Updated");
    }

    #[test]
    fn test_delete() {
        let dir = tempfile::tempdir().unwrap();
        let store = SkillStore::new(dir.path().join("skills"));

        store.add(make_skill("s1", "default", "Skill 1")).unwrap();
        store.add(make_skill("s2", "default", "Skill 2")).unwrap();
        store.delete("s1").unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "s2");
    }

    #[test]
    fn test_delete_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let store = SkillStore::new(dir.path().join("skills"));

        let result = store.delete("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let store = SkillStore::new(dir.path().join("skills"));

        store.add(make_skill("s1", "default", "Skill 1")).unwrap();
        let result = store.add(make_skill("s1", "default", "Skill 2"));
        assert!(result.is_err());
    }
}
