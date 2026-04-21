//! Tauri IPC command handlers.
//!
//! Each command handles a specific domain of IPC calls from the frontend.
//! Commands are registered in lib.rs via `invoke_handler`.

pub mod a2a_server;
pub mod activity;
pub mod channel;
pub mod collaboration;
pub mod remote_connection;
pub mod task;
pub mod task_suggestion;
pub mod thread;

use std::fs;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::context::ContextBuilder;
use crate::runtime::registry::RuntimeRegistry;
use crate::runtime::RuntimeType;
use crate::runtime::a2a::types::ConnectionMode;
use crate::storage::activity::{ActivityStore, ActivityType, create_entry};
use crate::workspace::manager::{AgentManager, AgentSummary, ManagerStatus, AgentHealthInfo};
use crate::workspace::identity::IdentitySummary;
use rusqlite::Connection;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Application state managed by Tauri.
///
/// Combines runtime registry, session state, agent workspace management,
/// and the SQLite database connection.
pub struct AppState {
    /// Agent runtime registry for managing agent runtimes (Claude Code, etc.)
    pub agent_runtime_registry: Mutex<RuntimeRegistry>,
    /// Current agent session state (session_id + process tracking)
    pub agent_session: Mutex<AgentSessionState>,
    /// Agent workspace manager for multi-Agent workspace isolation
    pub agent_manager: Mutex<AgentManager>,
    /// SQLite database connection for structured metadata.
    pub db_conn: Mutex<Connection>,
    /// A2A LAN server state (start/stop lifecycle).
    pub a2a_server: Mutex<a2a_server::A2AServerState>,
}

/// State tracking for an active agent session.
#[derive(Default)]
pub struct AgentSessionState {
    /// The current session ID (if any)
    pub session_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Activity logging helper (dual-write: JSONL + SQLite, best-effort)
// ---------------------------------------------------------------------------

/// Log an activity entry. Best-effort dual-write:
/// 1. Append to JSONL file (existing behavior)
/// 2. Insert into SQLite activity_log table (new)
///
/// Errors are logged but not propagated.
fn try_log_activity(
    workspace_root: &std::path::Path,
    db_conn: &Connection,
    activity_type: ActivityType,
    agent_id: Option<String>,
    summary: String,
    details: serde_json::Value,
) {
    let store = ActivityStore::new(workspace_root);
    let entry = create_entry(activity_type.clone(), agent_id.clone(), summary.clone(), details.clone());
    if let Err(e) = store.append(&entry) {
        log::warn!("[ActivityLog] Failed to log activity to JSONL: {}", e);
    }

    // Dual-write to SQLite
    let activity_type_str = serde_json::to_string(&activity_type)
        .unwrap_or_else(|_| "\"system\"".to_string())
        .trim_matches('"')
        .to_string();
    let db_row = crate::storage::db_helpers::ActivityLogRow {
        id: entry.id.clone(),
        timestamp: entry.timestamp.clone(),
        activity_type: activity_type_str,
        agent_id: entry.agent_id.clone(),
        workspace_id: entry.workspace_id.clone(),
        summary: entry.summary.clone(),
        details_json: serde_json::to_string(&entry.details).unwrap_or_else(|_| "{}".to_string()),
    };
    if let Err(e) = crate::storage::db_helpers::insert_activity(db_conn, &db_row) {
        log::warn!("[ActivityLog] Failed to log activity to SQLite: {}", e);
    }
}

// ---------------------------------------------------------------------------
// Workspace commands
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct InitWorkspaceResult {
    pub templates_created: Vec<String>,
    pub templates_skipped: Vec<String>,
    pub default_created: bool,
}

/// Initialize the workspace for the first time.
///
/// Creates the global workspace directory structure, templates,
/// and the default Agent.
#[tauri::command]
pub fn init_workspace(state: tauri::State<'_, AppState>) -> Result<InitWorkspaceResult, String> {
    let mut manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let result = manager
        .initialize_workspace()
        .map_err(|e| format!("init failed: {e}"))?;

    // Load the default agent into memory
    manager.load().map_err(|e| format!("load failed: {e}"))?;

    // Also load remote agents from database into memory
    {
        let db_conn = state
            .db_conn
            .lock()
            .map_err(|e| format!("db lock error: {e}"))?;
        let remote_rows = crate::storage::db_helpers::list_remote_agents(&db_conn)
            .map_err(|e| format!("remote agents load failed: {e}"))?;
        let remote_count = remote_rows.len();
        for row in remote_rows {
            let rt = crate::runtime::RuntimeType::Custom(
                if row.runtime_type.is_empty() { "remote-a2a".to_string() } else { row.runtime_type.clone() }
            );
            let cm = if row.connection_mode.starts_with("remote:") {
                let conn_id = row.connection_mode.trim_start_matches("remote:").to_string();
                crate::runtime::a2a::types::ConnectionMode::Remote { connection_id: conn_id }
            } else {
                crate::runtime::a2a::types::ConnectionMode::Local
            };
            manager.register_remote_agent(
                row.id,
                row.name,
                row.emoji,
                rt,
                cm,
            );
        }
        if remote_count > 0 {
            log::info!("[init_workspace] Loaded {} remote agents into memory", remote_count);
        }
    }

    Ok(InitWorkspaceResult {
        templates_created: result.templates_synced.created,
        templates_skipped: result.templates_synced.skipped,
        default_created: result.default_created,
    })
}

/// Get the current workspace and agent manager status.
#[tauri::command]
pub fn get_workspace_status(state: tauri::State<'_, AppState>) -> Result<ManagerStatus, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    Ok(manager.get_status())
}

/// Health check result for the workspace.
#[derive(Debug, Serialize)]
pub struct HealthCheckResult {
    /// Per-agent health info.
    pub agents: Vec<AgentHealthInfo>,
    /// Number of agents that were repaired.
    pub repaired: usize,
    /// Number of agents that are still unhealthy after repair attempt.
    pub still_unhealthy: usize,
}

/// Perform a workspace health check and optionally repair missing files/directories.
///
/// Scans all agent workspaces, reports missing items, and attempts to
/// recreate any missing directories. Does NOT recreate missing identity
/// or soul files (those require agent-specific content).
#[tauri::command]
pub fn health_check_workspace(
    state: tauri::State<'_, AppState>,
) -> Result<HealthCheckResult, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let status = manager.get_status();
    let mut repaired = 0usize;

    for health in &status.agents_health {
        if health.is_healthy {
            continue;
        }

        // Attempt repair: recreate workspace directories
        let workspace = manager
            .get_workspace(&health.agent_id)
            .ok_or_else(|| format!("agent not found: {}", health.agent_id))?;

        if let Err(e) = workspace.initialize() {
            log::warn!(
                "[HealthCheck] Failed to repair workspace for '{}': {}",
                health.agent_id,
                e
            );
        } else {
            log::info!(
                "[HealthCheck] Repaired workspace directories for '{}'",
                health.agent_id
            );
            repaired += 1;
        }
    }

    // Re-check status after repair
    let updated_status = manager.get_status();
    let unhealthy_after = updated_status
        .agents_health
        .iter()
        .filter(|h| !h.is_healthy)
        .count();

    Ok(HealthCheckResult {
        agents: updated_status.agents_health,
        repaired,
        still_unhealthy: unhealthy_after,
    })
}

// ---------------------------------------------------------------------------
// Agent CRUD commands
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    #[serde(default = "default_creature")]
    pub creature: String,
    #[serde(default = "default_vibe")]
    pub vibe: String,
    #[serde(default = "default_emoji")]
    pub emoji: String,
    pub avatar: Option<String>,
    /// SVG icon name from the icon registry (e.g. "Bot", "Rocket").
    #[serde(default)]
    pub icon: Option<String>,
    /// Runtime type for the agent (defaults to claude_code).
    #[serde(default)]
    pub runtime_type: Option<RuntimeType>,
    /// Connection mode for the agent (defaults to local).
    #[serde(default)]
    pub connection_mode: Option<ConnectionMode>,
}

fn default_creature() -> String {
    "AI".to_string()
}
fn default_vibe() -> String {
    "helpful".to_string()
}
fn default_emoji() -> String {
    "robot".to_string()
}

/// Create a new Agent with the specified parameters.
#[tauri::command]
pub fn create_agent(
    state: tauri::State<'_, AppState>,
    request: CreateAgentRequest,
) -> Result<AgentSummary, String> {
    let mut manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let agent = manager
        .create_agent(
            &request.name,
            &request.creature,
            &request.vibe,
            &request.emoji,
            request.avatar.as_deref(),
            request.icon.as_deref(),
            request.runtime_type.clone().unwrap_or_default(),
            request.connection_mode.clone().unwrap_or_default(),
        )
        .map_err(|e| format!("create failed: {e}"))?;

    let summary = agent.to_summary();

    // Log activity (dual-write: JSONL + SQLite)
    let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
    try_log_activity(
        manager.workspace_root(),
        &db_conn,
        ActivityType::AgentCreated,
        Some(summary.agent_id.clone()),
        format!("Agent \"{}\" created", summary.name),
        serde_json::json!({
            "name": summary.name,
            "emoji": summary.emoji,
        }),
    );

    Ok(summary)
}

/// List all available Agents.
#[tauri::command]
pub fn list_agents(state: tauri::State<'_, AppState>) -> Result<Vec<AgentSummary>, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    Ok(manager.list_agents())
}

/// Switch to a different Agent by ID.
#[tauri::command]
pub fn switch_agent(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<AgentSummary, String> {
    let mut manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let agent = manager
        .switch_agent(&agent_id)
        .map_err(|e| format!("switch failed: {e}"))?;

    Ok(agent.to_summary())
}

/// Get the currently active Agent.
#[tauri::command]
pub fn get_active_agent(state: tauri::State<'_, AppState>) -> Result<Option<AgentSummary>, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    Ok(manager.get_active_agent().map(|a| a.to_summary()))
}

/// Delete an Agent by ID.
#[tauri::command]
pub fn delete_agent(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<(), String> {
    let mut manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    manager
        .delete_agent(&agent_id)
        .map_err(|e| format!("delete failed: {e}"))?;

    // Log activity (dual-write: JSONL + SQLite)
    let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
    try_log_activity(
        manager.workspace_root(),
        &db_conn,
        ActivityType::AgentDeleted,
        Some(agent_id.clone()),
        format!("Agent \"{}\" deleted", agent_id),
        serde_json::json!({ "agent_id": agent_id }),
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Update Agent command
// ---------------------------------------------------------------------------

/// Request to update an existing Agent's mutable properties.
#[derive(Debug, Deserialize)]
pub struct UpdateAgentRequest {
    /// New display name (optional).
    pub name: Option<String>,
    /// New creature type (optional).
    pub creature: Option<String>,
    /// New vibe/personality (optional).
    pub vibe: Option<String>,
    /// New emoji (optional).
    pub emoji: Option<String>,
    /// New SVG icon name (optional, empty string clears it).
    pub icon: Option<String>,
}

/// Update an existing Agent's properties.
#[tauri::command]
pub fn update_agent(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    request: UpdateAgentRequest,
) -> Result<AgentSummary, String> {
    let mut manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let agent = manager
        .update_agent(
            &agent_id,
            request.name.as_deref(),
            request.creature.as_deref(),
            request.vibe.as_deref(),
            request.emoji.as_deref(),
            request.icon.as_deref(),
        )
        .map_err(|e| format!("update failed: {e}"))?;

    let summary = agent.to_summary();

    // Log activity (dual-write: JSONL + SQLite)
    let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
    try_log_activity(
        manager.workspace_root(),
        &db_conn,
        ActivityType::System,
        Some(agent_id.clone()),
        format!("Agent \"{}\" updated", summary.name),
        serde_json::json!({
            "name": summary.name,
            "emoji": summary.emoji,
        }),
    );

    Ok(summary)
}

// ---------------------------------------------------------------------------
// Identity commands
// ---------------------------------------------------------------------------

/// Get the identity of a specific Agent.
#[tauri::command]
pub fn get_agent_identity(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<IdentitySummary, String> {
    // Try local AgentManager first
    {
        let manager = state
            .agent_manager
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;

        if let Some(agent) = manager.get_agent(&agent_id) {
            return Ok(agent.identity.to_summary());
        }
    }

    // Fallback: check database for remote agents
    let db_conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("db lock error: {e}"))?;

    let row = crate::storage::db_helpers::get_agent(&db_conn, &agent_id)
        .map_err(|e| format!("db query error: {e}"))?
        .ok_or_else(|| format!("agent not found: {agent_id}"))?;

    let runtime_type = crate::runtime::RuntimeType::Custom(
        if row.runtime_type.is_empty() { "remote-a2a".to_string() } else { row.runtime_type }
    );
    let connection_mode = if row.connection_mode.starts_with("remote:") {
        let conn_id = row.connection_mode.trim_start_matches("remote:").to_string();
        crate::runtime::a2a::types::ConnectionMode::Remote { connection_id: conn_id }
    } else {
        crate::runtime::a2a::types::ConnectionMode::Local
    };

    Ok(IdentitySummary {
        agent_id: row.id,
        name: row.name,
        emoji: row.emoji,
        avatar: row.avatar_path,
        icon: None,
        creature: "Remote Agent".to_string(),
        vibe: "协作".to_string(),
        runtime_type,
        connection_mode,
    })
}

// ---------------------------------------------------------------------------
// Context commands
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct AgentContextResult {
    pub agent_id: String,
    pub system_prompt: String,
    pub has_user_context: bool,
    pub has_agent_instructions: bool,
    pub has_tool_instructions: bool,
    pub has_memory: bool,
    pub has_history: bool,
    pub context_prefix_length: usize,
}

/// Build the full context for an Agent (for debugging/preview).
#[tauri::command]
pub fn get_agent_context(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<AgentContextResult, String> {
    // Remote agents don't have local context files
    if agent_id.starts_with("remote:") {
        return Ok(AgentContextResult {
            agent_id,
            system_prompt: String::new(),
            has_user_context: false,
            has_agent_instructions: false,
            has_tool_instructions: false,
            has_memory: false,
            has_history: false,
            context_prefix_length: 0,
        });
    }

    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace_root = manager.workspace_root().to_path_buf();
    let builder = ContextBuilder::new(&workspace_root);
    let ctx = builder.build(&agent_id).map_err(|e| format!("context build failed: {e}"))?;

    let prefix_len = builder
        .build_context_prefix(&agent_id)
        .map(|p| p.len())
        .unwrap_or(0);

    Ok(AgentContextResult {
        agent_id: ctx.agent_id,
        system_prompt: ctx.system_prompt,
        has_user_context: ctx.user_context.is_some(),
        has_agent_instructions: ctx.agent_instructions.is_some(),
        has_tool_instructions: ctx.tool_instructions.is_some(),
        has_memory: ctx.memory.is_some(),
        has_history: ctx.history_summary.is_some(),
        context_prefix_length: prefix_len,
    })
}

// ---------------------------------------------------------------------------
// Agent + Runtime status command
// ---------------------------------------------------------------------------

/// Combined status of an agent: workspace info fused with runtime availability.
#[derive(Debug, Clone, Serialize)]
pub struct AgentWithRuntime {
    /// Agent workspace summary (from AgentManager).
    pub agent: AgentSummary,
    /// Runtime status string: "available" | "not-installed" | "unhealthy" | "detecting"
    pub runtime_status: String,
    /// Detected version (if runtime is available).
    pub runtime_version: Option<String>,
    /// Install hint (shown when runtime is not installed).
    pub runtime_install_hint: Option<String>,
    /// The runtime type this agent is configured to use.
    pub runtime_type: RuntimeType,
}

/// Get all agents with their runtime status fused together.
///
/// Scans registered runtimes and joins the result with the agent list
/// from the workspace manager. Each agent entry includes the runtime
/// availability information.
#[tauri::command]
pub fn get_agent_runtime_status(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AgentWithRuntime>, String> {
    // Get agent list from workspace manager
    let agents = {
        let manager = state
            .agent_manager
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;
        manager.list_agents()
    };

    // Get runtime status from registry (uses cached detection data)
    let runtimes = {
        let registry = state
            .agent_runtime_registry
            .lock()
            .map_err(|e| e.to_string())?;
        registry.list_all()
    };

    // Build a lookup map: runtime_id -> AgentRuntimeInfo
    // Currently each agent maps to "claude-code" runtime as the default.
    // In the future, agents may have different runtime assignments.
    let runtime_map: std::collections::HashMap<String, &crate::runtime::AgentRuntimeInfo> = runtimes
        .iter()
        .map(|rt| (rt.id.clone(), rt))
        .collect();

    // Fuse agent workspace info with runtime status
    let result: Vec<AgentWithRuntime> = agents
        .into_iter()
        .map(|agent| {
            // Look up the runtime by the agent's configured runtime_type
            let runtime = runtime_map.get(agent.runtime_type.runtime_id());

            AgentWithRuntime {
                runtime_status: runtime
                    .map(|rt| rt.status.clone())
                    .unwrap_or_else(|| "not-installed".to_string()),
                runtime_version: runtime.and_then(|rt| rt.version.clone()),
                runtime_install_hint: runtime.map(|rt| rt.install_hint.clone()),
                runtime_type: agent.runtime_type.clone(),
                agent,
            }
        })
        .collect();

    Ok(result)
}

// ---------------------------------------------------------------------------
// Workspace browsing commands
// ---------------------------------------------------------------------------

/// A single entry (file or directory) in a workspace directory listing.
#[derive(Debug, Clone, Serialize)]
pub struct DirectoryEntry {
    /// Name of the file or directory.
    pub name: String,
    /// Whether this is a directory (true) or file (false).
    pub is_dir: bool,
    /// File size in bytes (0 for directories).
    pub size: u64,
    /// Last modified timestamp (Unix epoch seconds).
    pub modified: u64,
}

/// Content of a file from the workspace.
#[derive(Debug, Clone, Serialize)]
pub struct FileContent {
    /// Full path to the file.
    pub path: String,
    /// File name.
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// MIME type hint (based on extension).
    pub mime_type: String,
    /// File content as string.
    pub content: String,
}

/// List directory entries in an agent's workspace.
#[tauri::command]
pub fn list_workspace_dir(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    subpath: Option<String>,
) -> Result<Vec<DirectoryEntry>, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace = manager
        .get_workspace(&agent_id)
        .ok_or_else(|| format!("agent not found: {agent_id}"))?;

    // Build the target path
    let target_path = match &subpath {
        Some(rel_path) => {
            // Security: prevent path traversal
            let clean = rel_path.replace("..", "");
            if clean.contains("..") {
                return Err("invalid path: traversal not allowed".to_string());
            }
            workspace.base_path().join(&clean)
        }
        None => workspace.base_path().to_path_buf(),
    };

    // Verify the target is within the workspace
    if !target_path.starts_with(workspace.base_path()) {
        return Err("invalid path: outside workspace".to_string());
    }

    if !target_path.is_dir() {
        return Err("not a directory".to_string());
    }

    let entries = fs::read_dir(&target_path).map_err(|e| format!("read dir failed: {e}"))?;

    let mut result: Vec<DirectoryEntry> = entries
        .filter_map(|entry: std::io::Result<std::fs::DirEntry>| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = metadata.is_dir();
            let size = if is_dir { 0 } else { metadata.len() };
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            Some(DirectoryEntry {
                name,
                is_dir,
                size,
                modified,
            })
        })
        .collect();

    // Sort: directories first, then files, both alphabetically
    result.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(result)
}

/// Read the content of a file from an agent's workspace.
#[tauri::command]
pub fn read_workspace_file(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    file_path: String,
) -> Result<FileContent, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace = manager
        .get_workspace(&agent_id)
        .ok_or_else(|| format!("agent not found: {agent_id}"))?;

    // Security: prevent path traversal
    let clean_path = file_path.replace("..", "");
    if clean_path.contains("..") {
        return Err("invalid path: traversal not allowed".to_string());
    }

    let full_path = workspace.base_path().join(&clean_path);

    // Verify the target is within the workspace
    if !full_path.starts_with(workspace.base_path()) {
        return Err("invalid path: outside workspace".to_string());
    }

    if !full_path.is_file() {
        return Err("not a file".to_string());
    }

    let metadata = fs::metadata(&full_path).map_err(|e| format!("metadata failed: {e}"))?;
    let content = fs::read_to_string(&full_path).map_err(|e| format!("read failed: {e}"))?;

    let name = full_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mime_type = mime_guess::from_path(&full_path)
        .first_or_octet_stream()
        .to_string();

    Ok(FileContent {
        path: full_path.to_string_lossy().to_string(),
        name,
        size: metadata.len(),
        mime_type,
        content,
    })
}

// ---------------------------------------------------------------------------
// Open in Finder command
// ---------------------------------------------------------------------------

/// Open an agent's workspace directory in the system file manager.
///
/// Resolves the agent's workspace path and opens it using the OS-native
/// file manager (Finder on macOS, Explorer on Windows).
#[tauri::command]
pub fn open_in_finder(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<(), String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace = manager
        .get_workspace(&agent_id)
        .ok_or_else(|| format!("agent not found: {agent_id}"))?;

    let path = workspace.base_path().to_path_buf();

    if !path.exists() {
        return Err("Workspace directory not found".to_string());
    }

    // Use Tauri's shell open to launch the system file manager
    // This is cross-platform: macOS Finder, Windows Explorer, Linux file manager
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open Finder: {e}"))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open Explorer: {e}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open file manager: {e}"))?;
    }

    log::info!("[OpenInFinder] Opened workspace for agent '{}': {}", agent_id, path.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// Original test command
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to AgentsZone.", name)
}

// ---------------------------------------------------------------------------
// Skill management commands
// ---------------------------------------------------------------------------

use crate::workspace::skill::{SkillStore, Skill, SkillType, SkillStatus, generate_skill_id};

/// Skill info returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct SkillInfo {
    pub id: String,
    pub agent_id: String,
    pub name: String,
    pub skill_type: String,
    pub config: serde_json::Value,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Skill> for SkillInfo {
    fn from(skill: Skill) -> Self {
        Self {
            id: skill.id,
            agent_id: skill.agent_id,
            name: skill.name,
            skill_type: skill.skill_type.to_string(),
            config: skill.config,
            status: skill.status.to_string(),
            created_at: skill.created_at,
            updated_at: skill.updated_at,
        }
    }
}

/// Request to add a new Skill.
#[derive(Debug, Deserialize)]
pub struct AddSkillRequest {
    pub name: String,
    pub skill_type: String,
    pub config: serde_json::Value,
}

/// Request to update an existing Skill.
#[derive(Debug, Deserialize)]
pub struct UpdateSkillRequest {
    pub name: Option<String>,
    pub skill_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub status: Option<String>,
}

/// Helper: get the SkillStore for a given agent.
fn get_skill_store(state: &tauri::State<'_, AppState>, agent_id: &str) -> Result<SkillStore, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace = manager
        .get_workspace(agent_id)
        .ok_or_else(|| format!("agent not found: {agent_id}"))?;

    Ok(SkillStore::new(workspace.skills_dir()))
}

/// List all Skills for a given Agent.
#[tauri::command]
pub fn list_skills(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<Vec<SkillInfo>, String> {
    let store = get_skill_store(&state, &agent_id)?;
    let skills = store.load_all().map_err(|e| format!("load skills failed: {e}"))?;
    Ok(skills.into_iter().map(SkillInfo::from).collect())
}

/// Add a new Skill to an Agent.
#[tauri::command]
pub fn add_skill(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    request: AddSkillRequest,
) -> Result<SkillInfo, String> {
    let store = get_skill_store(&state, &agent_id)?;

    let skill_type = match request.skill_type.as_str() {
        "MCP Server" | "mcp_server" => SkillType::McpServer,
        "Tool" | "tool" => SkillType::Tool,
        "Custom Command" | "custom_command" => SkillType::CustomCommand,
        other => return Err(format!("unknown skill type: {other}")),
    };

    let now = {
        let dur = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = dur.as_secs();
        let days = secs / 86400;
        let time_secs = secs % 86400;
        let hours = time_secs / 3600;
        let minutes = (time_secs % 3600) / 60;
        let seconds = time_secs % 60;
        let (y, m, d) = {
            let mut dd = days;
            let mut yr = 1970u64;
            loop {
                let diy = if (yr % 4 == 0 && yr % 100 != 0) || yr % 400 == 0 { 366 } else { 365 };
                if dd < diy { break; }
                dd -= diy;
                yr += 1;
            }
            let leap = (yr % 4 == 0 && yr % 100 != 0) || yr % 400 == 0;
            let md: [u64; 12] = if leap { [31,29,31,30,31,30,31,31,30,31,30,31] } else { [31,28,31,30,31,30,31,31,30,31,30,31] };
            let mut mo = 1u64;
            for &x in &md { if dd < x { break; } dd -= x; mo += 1; }
            (yr, mo, dd + 1)
        };
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, hours, minutes, seconds)
    };

    let skill = Skill {
        id: generate_skill_id(),
        agent_id: agent_id.clone(),
        name: request.name,
        skill_type,
        config: request.config,
        status: SkillStatus::Active,
        created_at: now.clone(),
        updated_at: now,
    };

    let added = store.add(skill).map_err(|e| format!("add skill failed: {e}"))?;
    Ok(SkillInfo::from(added))
}

/// Update an existing Skill.
#[tauri::command]
pub fn update_skill(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    skill_id: String,
    request: UpdateSkillRequest,
) -> Result<SkillInfo, String> {
    let store = get_skill_store(&state, &agent_id)?;

    let skill_type = request.skill_type.and_then(|s| match s.as_str() {
        "MCP Server" | "mcp_server" => Some(SkillType::McpServer),
        "Tool" | "tool" => Some(SkillType::Tool),
        "Custom Command" | "custom_command" => Some(SkillType::CustomCommand),
        _ => None,
    });

    let status = request.status.and_then(|s| match s.as_str() {
        "Active" | "active" => Some(SkillStatus::Active),
        "Inactive" | "inactive" => Some(SkillStatus::Inactive),
        "Error" | "error" => Some(SkillStatus::Error),
        "Connecting" | "connecting" => Some(SkillStatus::Connecting),
        _ => None,
    });

    let updated = store
        .update(&skill_id, request.name, skill_type, request.config, status)
        .map_err(|e| format!("update skill failed: {e}"))?;

    Ok(SkillInfo::from(updated))
}

/// Delete a Skill.
#[tauri::command]
pub fn delete_skill(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    skill_id: String,
) -> Result<(), String> {
    let store = get_skill_store(&state, &agent_id)?;
    store.delete(&skill_id).map_err(|e| format!("delete skill failed: {e}"))?;
    Ok(())
}

/// Get the status of a single Skill.
#[tauri::command]
pub fn get_skill_status(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    skill_id: String,
) -> Result<SkillInfo, String> {
    let store = get_skill_store(&state, &agent_id)?;
    let skill = store.get(&skill_id)
        .map_err(|e| format!("get skill failed: {e}"))?
        .ok_or_else(|| format!("skill not found: {skill_id}"))?;
    Ok(SkillInfo::from(skill))
}
