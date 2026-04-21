//! Agent Manager -- multi-Agent orchestration.
//!
//! Manages the lifecycle of Agents: creation, listing, switching,
//! and deletion. Each Agent gets an isolated workspace directory.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::agent::AgentWorkspace;
use super::identity::AgentIdentity;
use super::templates::WorkspaceTemplates;
use crate::runtime::RuntimeType;
use crate::runtime::a2a::types::ConnectionMode;

// ---------------------------------------------------------------------------
// Agent record
// ---------------------------------------------------------------------------

/// A fully loaded Agent with identity and workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique agent identifier (directory name).
    pub agent_id: String,
    /// Agent identity metadata.
    pub identity: AgentIdentity,
    /// Whether this Agent is enabled.
    pub enabled: bool,
    /// Session count (how many conversations this Agent has handled).
    pub session_count: u32,
}

impl Agent {
    /// Convert to a summary for API responses.
    pub fn to_summary(&self) -> AgentSummary {
        AgentSummary {
            agent_id: self.agent_id.clone(),
            name: self.identity.name.clone(),
            emoji: self.identity.emoji.clone(),
            avatar: self.identity.avatar.clone(),
            icon: self.identity.icon.clone(),
            enabled: self.enabled,
            session_count: self.session_count,
            runtime_type: self.identity.runtime_type.clone(),
            connection_mode: self.identity.connection_mode.clone(),
        }
    }
}

/// Lightweight Agent summary for frontend display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    pub agent_id: String,
    pub name: String,
    pub emoji: String,
    pub avatar: Option<String>,
    /// SVG icon name from the icon registry (e.g. "Bot", "Rocket").
    #[serde(default)]
    pub icon: Option<String>,
    pub enabled: bool,
    pub session_count: u32,
    /// The runtime type this agent is bound to.
    #[serde(default = "default_runtime_type")]
    pub runtime_type: RuntimeType,
    /// Connection mode: Local (default) or Remote.
    #[serde(default)]
    pub connection_mode: ConnectionMode,
}

fn default_runtime_type() -> RuntimeType {
    RuntimeType::ClaudeCode
}

// ---------------------------------------------------------------------------
// Manager
// ---------------------------------------------------------------------------

/// Manages multiple Agents with isolated workspaces.
///
/// The workspace root has this layout:
///
/// ```text
/// <workspace_root>/
/// ├── SOUL.md, USER.md, AGENTS.md, TOOLS.md   (global templates)
/// ├── memory/                                   (global memory)
/// └── agents/
///     ├── default/
///     │   ├── IDENTITY.md, SOUL.md
///     │   ├── conversations/, context/, output/, skills/, config/
///     └── <other-agent>/
///         └── ...
/// ```
pub struct AgentManager {
    /// Root workspace path.
    workspace_root: PathBuf,
    /// Path to the `agents/` subdirectory.
    agents_dir: PathBuf,
    /// Loaded agents, keyed by agent_id.
    agents: HashMap<String, Agent>,
    /// Currently active agent ID.
    active_agent_id: Option<String>,
}

impl AgentManager {
    /// Create a new AgentManager pointing at the given workspace root.
    ///
    /// Does NOT load agents from disk; call [`Self::load`] to populate.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root = workspace_root.into();
        let agents_dir = root.join("agents");
        Self {
            workspace_root: root,
            agents_dir,
            agents: HashMap::new(),
            active_agent_id: None,
        }
    }

    /// Get the workspace root path.
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Get the agents directory path.
    pub fn agents_dir(&self) -> &Path {
        &self.agents_dir
    }

    /// Get the channels directory path (workspace-level).
    pub fn channels_dir(&self) -> PathBuf {
        self.workspace_root.join("channels")
    }

    /// Get the currently active agent ID.
    pub fn active_agent_id(&self) -> Option<&str> {
        self.active_agent_id.as_deref()
    }

    /// Initialize the global workspace structure.
    ///
    /// Creates the workspace root, agents directory, and syncs global templates.
    /// This is typically called on first application launch.
    pub fn initialize_workspace(&self) -> Result<InitResult, ManagerError> {
        // Create root directory
        fs::create_dir_all(&self.workspace_root).map_err(|e| ManagerError::Io {
            path: self.workspace_root.clone(),
            source: e,
        })?;

        // Create agents directory
        fs::create_dir_all(&self.agents_dir).map_err(|e| ManagerError::Io {
            path: self.agents_dir.clone(),
            source: e,
        })?;

        // Sync global templates (SOUL.md, USER.md, etc.)
        let sync_result = WorkspaceTemplates::sync_global(&self.workspace_root)?;

        // Create the default Agent if it doesn't exist
        let default_dir = self.agents_dir.join("default");
        let default_created = if !default_dir.exists() {
            self.create_agent_internal(
                "default",
                "AgentsZone",
                "AI",
                "helpful",
                "robot",
                None,
                None,
                RuntimeType::ClaudeCode,
                ConnectionMode::Local,
            )?;
            true
        } else {
            false
        };

        Ok(InitResult {
            templates_synced: sync_result,
            default_created,
        })
    }

    /// Load all agents from disk.
    ///
    /// Scans the `agents/` directory and loads each Agent's identity.
    /// The first agent found becomes the active agent (or "default" if present).
    ///
    /// For each agent directory found, also performs a lightweight health check
    /// to ensure the workspace subdirectories exist (self-healing).
    pub fn load(&mut self) -> Result<(), ManagerError> {
        self.agents.clear();

        if !self.agents_dir.is_dir() {
            log::warn!(
                "[AgentManager] agents directory does not exist: {}",
                self.agents_dir.display()
            );
            return Ok(());
        }

        let entries = fs::read_dir(&self.agents_dir).map_err(|e| ManagerError::Io {
            path: self.agents_dir.clone(),
            source: e,
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| ManagerError::Io {
                path: self.agents_dir.clone(),
                source: e,
            })?;

            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let agent_id = entry.file_name().to_string_lossy().to_string();

            // Skip remote agent directories — they are managed by remote sync, not local workspace
            if agent_id.starts_with("remote:") {
                continue;
            }

            let identity = self.load_identity(&agent_id)?;

            // Self-heal: ensure workspace subdirectories exist for loaded agent
            let workspace = AgentWorkspace::new(&path);
            if !workspace.conversations_dir().exists() || !workspace.context_dir().exists() {
                log::info!(
                    "[AgentManager] Self-healing workspace directories for agent '{}'",
                    agent_id
                );
                if let Err(e) = workspace.initialize() {
                    log::warn!(
                        "[AgentManager] Failed to heal workspace for '{}': {}",
                        agent_id,
                        e
                    );
                }
            }

            let agent = Agent {
                agent_id: agent_id.clone(),
                identity,
                enabled: true,
                session_count: 0,
            };

            self.agents.insert(agent_id, agent);
        }

        // Set active agent: prefer "default", otherwise first loaded
        if self.agents.contains_key("default") {
            self.active_agent_id = Some("default".to_string());
        } else if let Some(first_id) = self.agents.keys().next() {
            self.active_agent_id = Some(first_id.clone());
        }

        Ok(())
    }

    /// Create a new Agent with the given parameters.
    ///
    /// Creates:
    /// 1. The Agent workspace directory with all subdirs
    /// 2. IDENTITY.md with the given metadata
    /// 3. SOUL.md with a personalized personality
    ///
    /// Returns the created Agent.
    pub fn create_agent(
        &mut self,
        name: &str,
        creature: &str,
        vibe: &str,
        emoji: &str,
        avatar: Option<&str>,
        icon: Option<&str>,
        runtime_type: RuntimeType,
        connection_mode: ConnectionMode,
    ) -> Result<&Agent, ManagerError> {
        let agent_id = name_to_id(name);

        if self.agents.contains_key(&agent_id) {
            return Err(ManagerError::AlreadyExists {
                agent_id: agent_id.clone(),
            });
        }

        let avatar_str = avatar.map(|s| s.to_string());
        let icon_str = icon.map(|s| s.to_string());
        self.create_agent_internal(&agent_id, name, creature, vibe, emoji, avatar_str, icon_str, runtime_type, connection_mode)?;

        let identity = self.load_identity(&agent_id)?;
        let agent = Agent {
            agent_id: agent_id.clone(),
            identity,
            enabled: true,
            session_count: 0,
        };

        // Set as default if this is the first agent
        if self.agents.is_empty() {
            self.active_agent_id = Some(agent_id.clone());
        }

        self.agents.insert(agent_id.clone(), agent);
        Ok(self.agents.get(&agent_id).unwrap())
    }

    /// Internal: create the Agent on disk (directory + templates).
    fn create_agent_internal(
        &self,
        agent_id: &str,
        name: &str,
        creature: &str,
        vibe: &str,
        emoji: &str,
        avatar: Option<String>,
        icon: Option<String>,
        runtime_type: RuntimeType,
        connection_mode: ConnectionMode,
    ) -> Result<(), ManagerError> {
        let agent_dir = self.agents_dir.join(agent_id);

        // Create workspace with all subdirs
        let workspace = AgentWorkspace::new(&agent_dir);
        workspace.initialize()?;

        // Create identity with runtime type and connection mode
        let mut identity = AgentIdentity::with_runtime_type(agent_id, name, creature, vibe, emoji, runtime_type);
        identity.avatar = avatar;
        identity.icon = icon;
        identity.connection_mode = connection_mode;
        identity
            .write_to_file(&workspace.identity_file())
            .map_err(|e| ManagerError::IdentityError {
                agent_id: agent_id.to_string(),
                source: e,
            })?;

        // Create personalized SOUL.md
        let soul_content = WorkspaceTemplates::agent_soul_template(name, emoji, vibe);
        WorkspaceTemplates::sync_agent(&agent_dir, &identity.to_identity_content(), &soul_content)?;

        Ok(())
    }

    /// Switch to a different agent by ID.
    pub fn switch_agent(&mut self, agent_id: &str) -> Result<&Agent, ManagerError> {
        if !self.agents.contains_key(agent_id) {
            return Err(ManagerError::NotFound {
                agent_id: agent_id.to_string(),
            });
        }
        self.active_agent_id = Some(agent_id.to_string());
        Ok(self.agents.get(agent_id).unwrap())
    }

    /// Register a remote agent in the in-memory map.
    ///
    /// Remote agents are synced from A2A connections and stored in the database.
    /// This method creates a lightweight in-memory entry so that `get_agent()` can
    /// find them without hitting the database.
    pub fn register_remote_agent(
        &mut self,
        agent_id: String,
        name: String,
        emoji: String,
        runtime_type: RuntimeType,
        connection_mode: ConnectionMode,
    ) {
        let identity = AgentIdentity {
            agent_id: agent_id.clone(),
            name,
            creature: "Remote Agent".to_string(),
            vibe: "协作".to_string(),
            emoji,
            avatar: None,
            icon: None,
            runtime_type,
            connection_mode,
        };
        self.agents.insert(agent_id.clone(), Agent {
            agent_id,
            identity,
            enabled: true,
            session_count: 0,
        });
    }

    /// Remove a remote agent from the in-memory map.
    pub fn unregister_remote_agent(&mut self, agent_id: &str) {
        self.agents.remove(agent_id);
    }

    /// Get an agent by ID.
    pub fn get_agent(&self, agent_id: &str) -> Option<&Agent> {
        self.agents.get(agent_id)
    }

    /// Get the currently active agent.
    pub fn get_active_agent(&self) -> Option<&Agent> {
        self.active_agent_id
            .as_ref()
            .and_then(|id| self.agents.get(id))
    }

    /// List all agents as summaries.
    pub fn list_agents(&self) -> Vec<AgentSummary> {
        let mut summaries: Vec<_> = self
            .agents
            .values()
            .filter(|a| a.enabled)
            .map(|a| a.to_summary())
            .collect();
        summaries.sort_by(|a, b| a.name.cmp(&b.name));
        summaries
    }

    /// Delete an agent by ID.
    ///
    /// Removes the agent from memory and deletes its workspace directory.
    /// Cannot delete the active agent.
    pub fn delete_agent(&mut self, agent_id: &str) -> Result<(), ManagerError> {
        if !self.agents.contains_key(agent_id) {
            return Err(ManagerError::NotFound {
                agent_id: agent_id.to_string(),
            });
        }

        if self.active_agent_id.as_deref() == Some(agent_id) {
            return Err(ManagerError::CannotDeleteActive {
                agent_id: agent_id.to_string(),
            });
        }

        // Delete workspace directory
        let agent_dir = self.agents_dir.join(agent_id);
        if agent_dir.exists() {
            fs::remove_dir_all(&agent_dir).map_err(|e| ManagerError::Io {
                path: agent_dir.clone(),
                source: e,
            })?;
        }

        self.agents.remove(agent_id);

        // If the deleted agent was the default, reassign to first remaining
        if self.agents.is_empty() {
            self.active_agent_id = None;
        }

        Ok(())
    }

    /// Get the AgentWorkspace handle for a specific agent.
    pub fn get_workspace(&self, agent_id: &str) -> Option<AgentWorkspace> {
        if self.agents.contains_key(agent_id) {
            Some(AgentWorkspace::new(self.agents_dir.join(agent_id)))
        } else {
            None
        }
    }

    /// Get the AgentWorkspace handle for the active agent.
    pub fn get_active_workspace(&self) -> Option<AgentWorkspace> {
        self.active_agent_id
            .as_ref()
            .map(|id| AgentWorkspace::new(self.agents_dir.join(id)))
    }

    /// Get manager status summary.
    pub fn get_status(&self) -> ManagerStatus {
        let agents_health: Vec<AgentHealthInfo> = self
            .agents
            .keys()
            .map(|agent_id| {
                let workspace = AgentWorkspace::new(self.agents_dir.join(agent_id));
                let health = Self::check_agent_health(&workspace);
                AgentHealthInfo {
                    agent_id: agent_id.clone(),
                    workspace_exists: workspace.exists(),
                    identity_file_exists: workspace.identity_file().exists(),
                    soul_file_exists: workspace.soul_file().exists(),
                    conversations_dir_exists: workspace.conversations_dir().exists(),
                    context_dir_exists: workspace.context_dir().exists(),
                    is_healthy: health.is_empty(),
                    missing_items: health,
                }
            })
            .collect();

        ManagerStatus {
            total_agents: self.agents.len(),
            enabled_agents: self.agents.values().filter(|a| a.enabled).count(),
            active_agent_id: self.active_agent_id.clone(),
            workspace_root: self.workspace_root.to_string_lossy().to_string(),
            agents_health,
        }
    }

    /// Check the health of a single agent workspace.
    ///
    /// Returns a list of missing items (empty = healthy).
    fn check_agent_health(workspace: &AgentWorkspace) -> Vec<String> {
        let mut missing = Vec::new();

        if !workspace.exists() {
            missing.push("workspace directory".to_string());
            return missing; // If base dir is gone, nothing else matters
        }

        if !workspace.identity_file().exists() {
            missing.push("IDENTITY.md".to_string());
        }
        if !workspace.soul_file().exists() {
            missing.push("SOUL.md".to_string());
        }
        if !workspace.conversations_dir().exists() {
            missing.push("conversations/".to_string());
        }
        if !workspace.context_dir().exists() {
            missing.push("context/".to_string());
        }

        missing
    }

    /// Load identity for an agent from disk.
    fn load_identity(&self, agent_id: &str) -> Result<AgentIdentity, ManagerError> {
        let identity_path = self.agents_dir.join(agent_id).join("IDENTITY.md");

        if identity_path.exists() {
            AgentIdentity::from_identity_file(&identity_path).map_err(|e| {
                ManagerError::IdentityError {
                    agent_id: agent_id.to_string(),
                    source: e,
                }
            })
        } else {
            Ok(AgentIdentity::default_for(agent_id))
        }
    }

    /// Reload a single agent's identity from disk.
    pub fn reload_identity(&mut self, agent_id: &str) -> Result<(), ManagerError> {
        let identity = self.load_identity(agent_id)?;
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.identity = identity;
            Ok(())
        } else {
            Err(ManagerError::NotFound {
                agent_id: agent_id.to_string(),
            })
        }
    }

    /// Update an existing Agent's mutable properties.
    ///
    /// Updates the identity fields on disk and in memory.
    /// Only the fields present in the update request are changed.
    /// Returns the updated Agent.
    pub fn update_agent(
        &mut self,
        agent_id: &str,
        name: Option<&str>,
        creature: Option<&str>,
        vibe: Option<&str>,
        emoji: Option<&str>,
        icon: Option<&str>,
    ) -> Result<&Agent, ManagerError> {
        let agent = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| ManagerError::NotFound {
                agent_id: agent_id.to_string(),
            })?;

        // Update identity fields
        if let Some(n) = name {
            agent.identity.name = n.to_string();
        }
        if let Some(c) = creature {
            agent.identity.creature = c.to_string();
        }
        if let Some(v) = vibe {
            agent.identity.vibe = v.to_string();
        }
        if let Some(e) = emoji {
            agent.identity.emoji = e.to_string();
        }
        if let Some(i) = icon {
            agent.identity.icon = if i.is_empty() {
                None
            } else {
                Some(i.to_string())
            };
        }

        // Write updated identity to disk
        let workspace = AgentWorkspace::new(self.agents_dir.join(agent_id));
        agent
            .identity
            .write_to_file(&workspace.identity_file())
            .map_err(|e| ManagerError::IdentityError {
                agent_id: agent_id.to_string(),
                source: e,
            })?;

        Ok(self.agents.get(agent_id).unwrap())
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Result of workspace initialization.
#[derive(Debug)]
pub struct InitResult {
    pub templates_synced: super::templates::SyncResult,
    pub default_created: bool,
}

/// Manager status summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerStatus {
    pub total_agents: usize,
    pub enabled_agents: usize,
    pub active_agent_id: Option<String>,
    pub workspace_root: String,
    /// Per-agent health information.
    #[serde(default)]
    pub agents_health: Vec<AgentHealthInfo>,
}

/// Health information for a single agent workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealthInfo {
    /// Agent ID.
    pub agent_id: String,
    /// Whether the workspace base directory exists.
    pub workspace_exists: bool,
    /// Whether IDENTITY.md exists.
    pub identity_file_exists: bool,
    /// Whether SOUL.md exists.
    pub soul_file_exists: bool,
    /// Whether conversations/ directory exists.
    pub conversations_dir_exists: bool,
    /// Whether context/ directory exists.
    pub context_dir_exists: bool,
    /// Whether the workspace is fully healthy (no missing items).
    pub is_healthy: bool,
    /// List of missing items (empty if healthy).
    pub missing_items: Vec<String>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a display name to an agent_id (lowercase, spaces to underscores).
fn name_to_id(name: &str) -> String {
    name.to_lowercase().replace(' ', "_")
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("agent not found: {agent_id}")]
    NotFound { agent_id: String },

    #[error("agent already exists: {agent_id}")]
    AlreadyExists { agent_id: String },

    #[error("cannot delete active agent: {agent_id}")]
    CannotDeleteActive { agent_id: String },

    #[error("identity error for {agent_id}: {source}")]
    IdentityError {
        agent_id: String,
        #[source]
        source: super::identity::IdentityError,
    },

    #[error("template error: {0}")]
    TemplateError(#[from] super::templates::TemplateError),

    #[error("workspace error: {0}")]
    WorkspaceError(#[from] super::agent::WorkspaceError),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_to_id() {
        assert_eq!(name_to_id("Claude Code"), "claude_code");
        assert_eq!(name_to_id("Default"), "default");
        assert_eq!(name_to_id("My Agent"), "my_agent");
    }

    #[test]
    fn test_initialize_creates_default() {
        let dir = tempfile::tempdir().unwrap();
        let manager = AgentManager::new(dir.path());
        let result = manager.initialize_workspace().unwrap();

        assert!(result.default_created);
        assert!(dir.path().join("SOUL.md").exists());
        assert!(dir.path().join("agents/default").exists());
        assert!(dir.path().join("agents/default/IDENTITY.md").exists());
        assert!(dir.path().join("agents/default/SOUL.md").exists());
        assert!(dir.path().join("agents/default/conversations").exists());
    }

    #[test]
    fn test_create_and_list_agents() {
        let dir = tempfile::tempdir().unwrap();
        let mut manager = AgentManager::new(dir.path());
        manager.initialize_workspace().unwrap();
        manager.load().unwrap();

        manager.create_agent("Claude", "AI", "sharp", "sparkles", None, None, RuntimeType::ClaudeCode, ConnectionMode::Local).unwrap();
        manager.create_agent("Codex", "AI", "calm", "code", None, None, RuntimeType::Codex, ConnectionMode::Local).unwrap();

        let agents = manager.list_agents();
        assert_eq!(agents.len(), 3); // default + claude + codex
    }

    #[test]
    fn test_switch_agent() {
        let dir = tempfile::tempdir().unwrap();
        let mut manager = AgentManager::new(dir.path());
        manager.initialize_workspace().unwrap();
        manager.load().unwrap();

        manager.create_agent("Claude", "AI", "sharp", "sparkles", None, None, RuntimeType::ClaudeCode, ConnectionMode::Local).unwrap();

        let agent = manager.switch_agent("claude").unwrap();
        assert_eq!(agent.agent_id, "claude");
        assert_eq!(manager.active_agent_id(), Some("claude"));
    }

    #[test]
    fn test_delete_agent() {
        let dir = tempfile::tempdir().unwrap();
        let mut manager = AgentManager::new(dir.path());
        manager.initialize_workspace().unwrap();
        manager.load().unwrap();

        manager.create_agent("ToDelete", "AI", "calm", "x", None, None, RuntimeType::ClaudeCode, ConnectionMode::Local).unwrap();

        // Switch away first so it's not active
        manager.switch_agent("default").unwrap();
        manager.delete_agent("todelete").unwrap();

        assert!(manager.get_agent("todelete").is_none());
        assert!(!dir.path().join("agents/todelete").exists());
    }

    #[test]
    fn test_cannot_delete_active_agent() {
        let dir = tempfile::tempdir().unwrap();
        let mut manager = AgentManager::new(dir.path());
        manager.initialize_workspace().unwrap();
        manager.load().unwrap();

        let result = manager.delete_agent("default");
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_agent_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let mut manager = AgentManager::new(dir.path());
        manager.initialize_workspace().unwrap();
        manager.load().unwrap();

        // Try to create "Default" which normalizes to "default"
        let result = manager.create_agent("Default", "AI", "helpful", "robot", None, None, RuntimeType::ClaudeCode, ConnectionMode::Local);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_disk() {
        let dir = tempfile::tempdir().unwrap();

        // First manager: initialize and create an agent
        let mut m1 = AgentManager::new(dir.path());
        m1.initialize_workspace().unwrap();
        m1.load().unwrap();
        m1.create_agent("Claude", "AI", "sharp", "sparkles", None, None, RuntimeType::ClaudeCode, ConnectionMode::Local).unwrap();

        // Second manager: should load existing agents
        let mut m2 = AgentManager::new(dir.path());
        m2.load().unwrap();

        assert_eq!(m2.list_agents().len(), 2);
        assert!(m2.get_agent("claude").is_some());
    }
}
