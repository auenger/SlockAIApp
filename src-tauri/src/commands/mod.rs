//! Tauri IPC command handlers.
//!
//! Each command handles a specific domain of IPC calls from the frontend.
//! Commands are registered in lib.rs via `invoke_handler`.

pub mod channel;
pub mod thread;

use std::fs;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::context::ContextBuilder;
use crate::runtime::registry::RuntimeRegistry;
use crate::workspace::manager::{AgentManager, AgentSummary, ManagerStatus};
use crate::workspace::identity::IdentitySummary;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Application state managed by Tauri.
///
/// Combines runtime registry, session state, and agent workspace management.
pub struct AppState {
    /// Agent runtime registry for managing agent runtimes (Claude Code, etc.)
    pub agent_runtime_registry: Mutex<RuntimeRegistry>,
    /// Current agent session state (session_id + process tracking)
    pub agent_session: Mutex<AgentSessionState>,
    /// Agent workspace manager for multi-Agent workspace isolation
    pub agent_manager: Mutex<AgentManager>,
}

/// State tracking for an active agent session.
#[derive(Default)]
pub struct AgentSessionState {
    /// The current session ID (if any)
    pub session_id: Option<String>,
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
        )
        .map_err(|e| format!("create failed: {e}"))?;

    Ok(agent.to_summary())
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

    Ok(())
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
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let agent = manager
        .get_agent(&agent_id)
        .ok_or_else(|| format!("agent not found: {agent_id}"))?;

    Ok(agent.identity.to_summary())
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
            // Currently all agents use the "claude-code" runtime.
            // This will be configurable per-agent in the future.
            let runtime = runtime_map.get("claude-code");

            AgentWithRuntime {
                runtime_status: runtime
                    .map(|rt| rt.status.clone())
                    .unwrap_or_else(|| "not-installed".to_string()),
                runtime_version: runtime.and_then(|rt| rt.version.clone()),
                runtime_install_hint: runtime.map(|rt| rt.install_hint.clone()),
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
// Original test command
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to AgentsZone.", name)
}
