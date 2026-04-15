//! Tauri IPC commands for remote A2A connection management.
//!
//! Provides CRUD operations, health checking, and testing for
//! remote A2A endpoint connections.

use serde::{Deserialize, Serialize};

use crate::runtime::a2a::remote::RemoteConnectionManager;
use crate::runtime::a2a::types::{AgentCard, AuthType, ConnectionStatus, RemoteConnection};
use crate::commands::AppState;

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

    RemoteConnectionManager::health_check_all(&db_conn);

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
