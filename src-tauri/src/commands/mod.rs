/// Tauri IPC command handlers.
///
/// Each command handles a specific domain of IPC calls from the frontend.
/// Commands are registered in lib.rs via `invoke_handler`.

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
// Original test command
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to AgentsZone.", name)
}
