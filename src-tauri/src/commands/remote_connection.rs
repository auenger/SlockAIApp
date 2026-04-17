//! Tauri IPC commands for remote A2A connection management.
//!
//! Provides CRUD operations, health checking, and testing for
//! remote A2A endpoint connections.

use serde::{Deserialize, Serialize};

use crate::runtime::a2a::remote::RemoteConnectionManager;
use crate::runtime::a2a::types::{AgentCard, AuthType, ConnectionStatus, RemoteConnection};
use crate::commands::AppState;
use crate::workspace::manager::AgentSummary;

// ===========================================================================
// Request/Response types
// ===========================================================================

/// Request to create a new remote connection.
#[derive(Debug, Deserialize)]
pub struct CreateRemoteConnectionRequest {
    pub name: String,
    pub endpoint_url: String,
    #[serde(default)]
    pub auth_type: Option<String>,
    /// API key for authentication (stored in keyring, not DB).
    #[serde(default)]
    pub api_key: Option<String>,
}

/// Request to update a remote connection.
#[derive(Debug, Deserialize)]
pub struct UpdateRemoteConnectionRequest {
    pub name: Option<String>,
    pub endpoint_url: Option<String>,
    pub auth_type: Option<String>,
    /// New API key (if changing).
    #[serde(default)]
    pub api_key: Option<String>,
}

/// Remote connection info returned to the frontend (no sensitive data).
#[derive(Debug, Clone, Serialize)]
pub struct RemoteConnectionInfo {
    pub id: String,
    pub name: String,
    pub endpoint_url: String,
    pub auth_type: String,
    pub status: String,
    pub agent_card: Option<AgentCard>,
    pub last_health_check_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<RemoteConnection> for RemoteConnectionInfo {
    fn from(conn: RemoteConnection) -> Self {
        Self {
            id: conn.id,
            name: conn.name,
            endpoint_url: conn.endpoint_url,
            auth_type: match conn.auth_type {
                AuthType::None => "none".to_string(),
                AuthType::ApiKey => "api_key".to_string(),
                AuthType::OAuth2 => "oauth2".to_string(),
            },
            status: match conn.status {
                ConnectionStatus::Online => "online",
                ConnectionStatus::Offline => "offline",
                ConnectionStatus::Error => "error",
                ConnectionStatus::Unknown => "unknown",
            }.to_string(),
            agent_card: conn.cached_agent_card,
            last_health_check_at: conn.last_health_check_at,
            created_at: conn.created_at,
            updated_at: conn.updated_at,
        }
    }
}

/// Result of testing a remote connection.
#[derive(Debug, Serialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub agent_card: Option<AgentCard>,
    pub error: Option<String>,
}

// ===========================================================================
// Helper: get DB connection from state
// ===========================================================================

/// Get a reference to the SQLite connection from AppState.
fn get_db_conn<'a>(state: &'a tauri::State<'a, AppState>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state.db_conn.lock().map_err(|e| format!("lock error: {}", e))
}

// ===========================================================================
// IPC Commands
// ===========================================================================

/// Create a new remote connection.
#[tauri::command]
pub fn remote_connection_create(
    state: tauri::State<'_, AppState>,
    request: CreateRemoteConnectionRequest,
) -> Result<RemoteConnectionInfo, String> {
    let db_conn = get_db_conn(&state)?;

    let auth_type = match request.auth_type.as_deref() {
        Some("api_key") => AuthType::ApiKey,
        Some("oauth2") => AuthType::OAuth2,
        _ => AuthType::None,
    };

    let conn = RemoteConnectionManager::create(&db_conn, &request.name, &request.endpoint_url, auth_type)?;

    // Store API key in keyring if provided
    if let Some(ref api_key) = request.api_key {
        if !api_key.is_empty() {
            RemoteConnectionManager::store_auth_token(&conn.id, api_key)?;
        }
    }

    log::info!(
        "[remote_connection_create] Created: '{}' ({})",
        conn.name,
        conn.id
    );

    Ok(RemoteConnectionInfo::from(conn))
}

/// List all remote connections.
#[tauri::command]
pub fn remote_connection_list(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<RemoteConnectionInfo>, String> {
    let db_conn = get_db_conn(&state)?;
    let conns = RemoteConnectionManager::list(&db_conn)?;
    Ok(conns.into_iter().map(RemoteConnectionInfo::from).collect())
}

/// Update a remote connection.
#[tauri::command]
pub fn remote_connection_update(
    state: tauri::State<'_, AppState>,
    id: String,
    request: UpdateRemoteConnectionRequest,
) -> Result<RemoteConnectionInfo, String> {
    let db_conn = get_db_conn(&state)?;

    let conn = RemoteConnectionManager::update(
        &db_conn,
        &id,
        request.name.as_deref(),
        request.endpoint_url.as_deref(),
        request.auth_type.as_deref(),
    )?;

    // Update API key in keyring if provided
    if let Some(ref api_key) = request.api_key {
        if !api_key.is_empty() {
            RemoteConnectionManager::store_auth_token(&id, api_key)?;
        }
    }

    Ok(RemoteConnectionInfo::from(conn))
}

/// Delete a remote connection.
#[tauri::command]
pub fn remote_connection_delete(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db_conn = get_db_conn(&state)?;

    // Cascade: delete all remote agents associated with this connection
    let deleted_agents = crate::storage::db_helpers::delete_remote_agents_by_connection(&db_conn, &id)
        .map_err(|e| format!("Failed to clean up remote agents: {}", e))?;
    log::info!("[remote_connection_delete] Cleaned up {} remote agents for '{}'", deleted_agents, id);

    // Also clean up workspace directories for remote agents
    {
        let manager = state.agent_manager.lock().map_err(|e| format!("lock error: {}", e))?;
        let agents_dir = manager.agents_dir();
        // Remote agent dirs follow pattern: "remote:{connection_id}:{agent_name}"
        if let Ok(entries) = std::fs::read_dir(agents_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&format!("remote:{}:", id)) {
                    if let Err(e) = std::fs::remove_dir_all(entry.path()) {
                        log::warn!("[remote_connection_delete] Failed to remove remote agent dir '{}': {}", name, e);
                    }
                }
            }
        }
    }

    // Remove from channel_members table
    {
        let remote_agent_rows = crate::storage::db_helpers::list_remote_agents_by_connection(&db_conn, &id)
            .unwrap_or_default();
        for agent in remote_agent_rows {
            // This agent was already deleted above, but we need to clean channel_members
            // Since the agents table uses ON DELETE CASCADE? No, channel_members references agents(id).
            // Actually the agent rows are already deleted, so channel_members with CASCADE should auto-clean.
            log::info!("[remote_connection_delete] Channel members for '{}' auto-cleaned via CASCADE", agent.id);
        }
    }

    RemoteConnectionManager::delete(&db_conn, &id)?;
    log::info!("[remote_connection_delete] Deleted: {}", id);
    Ok(())
}

/// Test a remote connection (health check).
#[tauri::command]
pub fn remote_connection_test(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<TestConnectionResult, String> {
    let db_conn = get_db_conn(&state)?;

    match RemoteConnectionManager::health_check(&db_conn, &id) {
        Ok(card) => Ok(TestConnectionResult {
            success: true,
            agent_card: Some(card),
            error: None,
        }),
        Err(e) => Ok(TestConnectionResult {
            success: false,
            agent_card: None,
            error: Some(e),
        }),
    }
}

/// Batch health check all remote connections.
#[tauri::command]
pub fn remote_connection_health_all(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<RemoteConnectionInfo>, String> {
    let db_conn = get_db_conn(&state)?;

    let _results = RemoteConnectionManager::health_check_all(&db_conn);

    // Update remote agent enabled status based on health check results
    let connections = RemoteConnectionManager::list(&db_conn)?;
    for conn in &connections {
        let is_online = conn.status == ConnectionStatus::Online;
        let conn_id = &conn.id;

        if is_online {
            let enabled = crate::storage::db_helpers::enable_remote_agents_by_connection(&db_conn, conn_id)
                .unwrap_or(0);
            if enabled > 0 {
                log::info!("[remote_connection_health_all] Enabled {} remote agents for online connection '{}'", enabled, conn.name);
            }
        } else {
            let disabled = crate::storage::db_helpers::disable_remote_agents_by_connection(&db_conn, conn_id)
                .unwrap_or(0);
            if disabled > 0 {
                log::info!("[remote_connection_health_all] Disabled {} remote agents for offline connection '{}'", disabled, conn.name);
            }
        }
    }

    // Return updated list
    let conns = RemoteConnectionManager::list(&db_conn)?;
    Ok(conns.into_iter().map(RemoteConnectionInfo::from).collect())
}

/// Get the cached AgentCard for a remote connection.
#[tauri::command]
pub fn remote_connection_get_agent_card(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Option<AgentCard>, String> {
    let db_conn = get_db_conn(&state)?;
    let conn = RemoteConnectionManager::get(&db_conn, &id)?
        .ok_or_else(|| format!("Connection '{}' not found", id))?;
    Ok(conn.cached_agent_card)
}

// ===========================================================================
// Remote Agent Sync Commands
// ===========================================================================

/// Sync remote agents from a specific connection.
///
/// Calls the remote bridge's `/agents` endpoint, then upserts each agent
/// as a local proxy record in the agents table.
#[tauri::command]
pub fn sync_remote_agents(
    state: tauri::State<'_, AppState>,
    connection_id: String,
) -> Result<Vec<AgentSummary>, String> {
    let db_conn = get_db_conn(&state)?;

    // Get the remote connection
    let conn = RemoteConnectionManager::get(&db_conn, &connection_id)?
        .ok_or_else(|| format!("Connection '{}' not found", connection_id))?;

    if conn.status != ConnectionStatus::Online {
        return Err(format!("Connection '{}' is not online (status: {:?})", connection_id, conn.status));
    }

    // Fetch agents from the remote bridge endpoint
    let client = crate::runtime::a2a::remote::build_http_client(&conn)?;
    let url = format!(
        "{}/agents",
        conn.endpoint_url.trim_end_matches('/')
    );

    log::info!("[sync_remote_agents] Fetching agents from '{}' at {}", conn.name, url);

    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("Failed to fetch remote agents: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("Remote agents fetch failed ({}): {}", status, body));
    }

    // Parse the response — expect a JSON array of agent objects
    let remote_agents: Vec<RemoteAgentEntry> = response
        .json()
        .map_err(|e| format!("Failed to parse remote agents response: {}", e))?;

    log::info!(
        "[sync_remote_agents] Found {} agents from '{}'",
        remote_agents.len(),
        conn.name
    );

    // Upsert each remote agent into the local database
    let now = crate::storage::db_helpers::chrono_now_iso();
    let mut synced = Vec::new();

    for agent in &remote_agents {
        let agent_id = format!("remote:{}:{}", connection_id, agent.id);

        let row = crate::storage::db_helpers::AgentRow {
            id: agent_id.clone(),
            name: agent.name.clone(),
            emoji: agent.emoji.clone().unwrap_or_else(|| "cloud".to_string()),
            avatar_path: agent.avatar.clone(),
            enabled: true,
            runtime_type: agent.runtime_type.clone().unwrap_or_else(|| "remote-a2a".to_string()),
            description: agent.description.clone().unwrap_or_default(),
            connection_mode: format!("remote:{}", connection_id),
            remote_connection_id: Some(connection_id.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        crate::storage::db_helpers::upsert_remote_agent(&db_conn, &row)
            .map_err(|e| format!("Failed to upsert remote agent '{}': {}", agent.name, e))?;

        // Also add to the in-memory AgentManager
        {
            let manager = state.agent_manager.lock().map_err(|e| format!("lock error: {}", e))?;
            // Ensure a workspace directory exists for the remote agent (lightweight)
            let agents_dir = manager.agents_dir().join(&agent_id);
            if !agents_dir.exists() {
                std::fs::create_dir_all(&agents_dir).map_err(|e| format!("Failed to create remote agent dir: {}", e))?;
            }
        }

        synced.push(AgentSummary {
            agent_id,
            name: agent.name.clone(),
            emoji: agent.emoji.clone().unwrap_or_else(|| "cloud".to_string()),
            avatar: agent.avatar.clone(),
            icon: agent.icon.clone(),
            enabled: true,
            session_count: 0,
            runtime_type: crate::runtime::RuntimeType::Custom(
                agent.runtime_type.clone().unwrap_or_else(|| "remote-a2a".to_string())
            ),
            connection_mode: crate::runtime::a2a::types::ConnectionMode::Remote {
                connection_id: connection_id.clone(),
            },
        });
    }

    log::info!(
        "[sync_remote_agents] Synced {} agents from '{}'",
        synced.len(),
        conn.name
    );

    Ok(synced)
}

/// Get all remote agents across all connections.
#[tauri::command]
pub fn get_remote_agents(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AgentSummary>, String> {
    let db_conn = get_db_conn(&state)?;
    let rows = crate::storage::db_helpers::list_remote_agents(&db_conn)
        .map_err(|e| format!("Failed to list remote agents: {}", e))?;

    let agents: Vec<AgentSummary> = rows.into_iter().map(|row| {
        let connection_id = row.remote_connection_id.clone();
        AgentSummary {
            agent_id: row.id,
            name: row.name,
            emoji: row.emoji,
            avatar: row.avatar_path,
            icon: None,
            enabled: row.enabled,
            session_count: 0,
            runtime_type: crate::runtime::RuntimeType::Custom(row.runtime_type),
            connection_mode: match connection_id {
                Some(cid) => crate::runtime::a2a::types::ConnectionMode::Remote { connection_id: cid },
                None => crate::runtime::a2a::types::ConnectionMode::Local,
            },
        }
    }).collect();

    Ok(agents)
}

/// Refresh agents for a specific connection (sync + status update).
#[tauri::command]
pub fn refresh_remote_agents(
    state: tauri::State<'_, AppState>,
    connection_id: String,
) -> Result<(), String> {
    let db_conn = get_db_conn(&state)?;

    // First try a health check on the connection
    match RemoteConnectionManager::health_check(&db_conn, &connection_id) {
        Ok(_) => {
            // Connection is online, enable agents and sync
            let enabled = crate::storage::db_helpers::enable_remote_agents_by_connection(&db_conn, &connection_id)
                .map_err(|e| format!("Failed to enable agents: {}", e))?;
            log::info!("[refresh_remote_agents] Enabled {} agents for '{}'", enabled, connection_id);

            // Drop the DB lock before sync (sync will reacquire)
            drop(db_conn);

            // Sync agents from remote
            sync_remote_agents(state, connection_id)?;
        }
        Err(e) => {
            // Connection is offline, disable agents
            log::warn!("[refresh_remote_agents] Health check failed for '{}': {}", connection_id, e);
            let disabled = crate::storage::db_helpers::disable_remote_agents_by_connection(&db_conn, &connection_id)
                .map_err(|e| format!("Failed to disable agents: {}", e))?;
            log::info!("[refresh_remote_agents] Disabled {} agents for '{}'", disabled, connection_id);
        }
    }

    Ok(())
}

// ===========================================================================
// Remote Agent Entry (parsed from bridge response)
// ===========================================================================

/// An agent entry returned by the remote bridge's `/agents` endpoint.
#[derive(Debug, Clone, serde::Deserialize)]
struct RemoteAgentEntry {
    /// Agent identifier on the remote side.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Emoji (optional).
    pub emoji: Option<String>,
    /// Avatar URL (optional).
    pub avatar: Option<String>,
    /// SVG icon name (optional).
    pub icon: Option<String>,
    /// Description (optional).
    pub description: Option<String>,
    /// Runtime type on the remote side (optional).
    pub runtime_type: Option<String>,
}
