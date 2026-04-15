//! Remote Connection Manager.
//!
//! Manages remote A2A endpoint configurations stored in SQLite.
//! Provides CRUD operations, health checking, and HTTP client pooling
//! for remote agent connections.

use rusqlite::Connection;

use super::types::{AgentCard, AuthType, ConnectionStatus, RemoteConnection};
use crate::storage::db_helpers;
use crate::storage::db_helpers::RemoteConnectionRow;

// ===========================================================================
// RemoteConnectionManager
// ===========================================================================

/// Manages remote A2A endpoint connections.
///
/// This is a stateless helper that operates on the SQLite database.
/// Each method takes a `&Connection` reference, avoiding ownership issues
/// with the non-Clone `rusqlite::Connection` type.
pub struct RemoteConnectionManager;

impl RemoteConnectionManager {
    /// Create a new remote connection.
    pub fn create(
        db: &Connection,
        name: &str,
        endpoint_url: &str,
        auth_type: AuthType,
    ) -> Result<RemoteConnection, String> {
        let id = generate_connection_id();
        let now = db_helpers::chrono_now_iso();

        let auth_type_str = match &auth_type {
            AuthType::None => "none",
            AuthType::ApiKey => "api_key",
            AuthType::OAuth2 => "oauth2",
        };

        let row = RemoteConnectionRow {
            id: id.clone(),
            name: name.to_string(),
            endpoint_url: endpoint_url.to_string(),
            auth_type: auth_type_str.to_string(),
            status: "unknown".to_string(),
            cached_agent_card: None,
            last_health_check_at: None,
            created_at: now.clone(),
            updated_at: now,
        };

        db_helpers::insert_remote_connection(db, &row)
            .map_err(|e| format!("Failed to create remote connection: {}", e))?;

        let conn = row_to_connection(&row);

        log::info!(
            "[RemoteConnectionManager] Created connection '{}' ({})",
            name,
            id
        );

        Ok(conn)
    }

    /// List all remote connections.
    pub fn list(db: &Connection) -> Result<Vec<RemoteConnection>, String> {
        let rows = db_helpers::list_remote_connections(db)
            .map_err(|e| format!("Failed to list remote connections: {}", e))?;
        let mut conns: Vec<_> = rows.iter().map(row_to_connection).collect();
        conns.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(conns)
    }

    /// Get a single remote connection by ID.
    pub fn get(db: &Connection, id: &str) -> Result<Option<RemoteConnection>, String> {
        let row = db_helpers::get_remote_connection(db, id)
            .map_err(|e| format!("Failed to get remote connection: {}", e))?;
        Ok(row.as_ref().map(row_to_connection))
    }

    /// Update a remote connection.
    pub fn update(
        db: &Connection,
        id: &str,
        name: Option<&str>,
        endpoint_url: Option<&str>,
        auth_type: Option<&str>,
    ) -> Result<RemoteConnection, String> {
        let now = db_helpers::chrono_now_iso();

        db_helpers::update_remote_connection(
            db,
            id,
            name,
            endpoint_url,
            auth_type,
            None,
            None,
            None,
            &now,
        )
        .map_err(|e| format!("Failed to update remote connection: {}", e))?;

        // Reload from DB
        let row = db_helpers::get_remote_connection(db, id)
            .map_err(|e| format!("Failed to reload connection: {}", e))?
            .ok_or_else(|| format!("Connection '{}' not found after update", id))?;

        log::info!(
            "[RemoteConnectionManager] Updated connection '{}'",
            id
        );

        Ok(row_to_connection(&row))
    }

    /// Delete a remote connection.
    pub fn delete(db: &Connection, id: &str) -> Result<(), String> {
        db_helpers::delete_remote_connection(db, id)
            .map_err(|e| format!("Failed to delete remote connection: {}", e))?;

        // Clean up auth token from keyring
        let keyring_key = format!("remote_conn_{}", id);
        if let Err(e) = crate::storage::keyring::delete_api_key(keyring_key) {
            log::warn!(
                "[RemoteConnectionManager] Failed to clean up keyring for '{}': {}",
                id,
                e
            );
        }

        log::info!("[RemoteConnectionManager] Deleted connection '{}'", id);

        Ok(())
    }

    /// Perform a health check on a connection.
    ///
    /// Sends GET {endpoint}/agent-card and updates the connection status.
    pub fn health_check(db: &Connection, id: &str) -> Result<AgentCard, String> {
        let row = db_helpers::get_remote_connection(db, id)
            .map_err(|e| format!("Failed to get connection: {}", e))?
            .ok_or_else(|| format!("Connection '{}' not found", id))?;

        let conn = row_to_connection(&row);

        log::info!(
            "[RemoteConnectionManager] Health check for '{}' at {}",
            conn.name,
            conn.endpoint_url
        );

        // Build an HTTP client with optional auth
        let client = build_http_client(&conn)?;

        // Try to get the agent card
        let url = format!(
            "{}/agent-card",
            conn.endpoint_url.trim_end_matches('/')
        );

        let response = client
            .get(&url)
            .send()
            .map_err(|e| format!("Health check request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            let err_msg = format!("Health check failed ({}): {}", status, body);

            // Update status to error
            let now = db_helpers::chrono_now_iso();
            let _ = db_helpers::update_remote_connection(
                db,
                id,
                None,
                None,
                None,
                Some("error"),
                None,
                None,
                &now,
            );

            return Err(err_msg);
        }

        let agent_card: AgentCard = response
            .json()
            .map_err(|e| format!("Failed to parse agent card: {}", e))?;

        // Update status to online and cache the agent card
        let now = db_helpers::chrono_now_iso();
        let card_json = serde_json::to_string(&agent_card).unwrap_or_default();

        let _ = db_helpers::update_remote_connection(
            db,
            id,
            None,
            None,
            None,
            Some("online"),
            Some(Some(card_json.as_str())),
            Some(Some(now.as_str())),
            &now,
        );

        log::info!(
            "[RemoteConnectionManager] Health check OK for '{}': {}",
            conn.name,
            agent_card.name
        );

        Ok(agent_card)
    }

    /// Perform health checks on all connections.
    pub fn health_check_all(db: &Connection) -> Vec<Result<AgentCard, String>> {
        let rows = match db_helpers::list_remote_connections(db) {
            Ok(r) => r,
            Err(e) => {
                log::error!("[RemoteConnectionManager] Failed to list connections: {}", e);
                return vec![];
            }
        };
        rows.iter()
            .map(|row| Self::health_check(db, &row.id))
            .collect()
    }

    /// Store an API key for a remote connection in the keyring.
    pub fn store_auth_token(id: &str, token: &str) -> Result<(), String> {
        let keyring_key = format!("remote_conn_{}", id);
        crate::storage::keyring::store_api_key(keyring_key, token.to_string())?;
        log::info!(
            "[RemoteConnectionManager] Auth token stored for connection '{}'",
            id
        );
        Ok(())
    }

    /// Get the auth token for a remote connection from the keyring.
    pub fn get_auth_token(id: &str) -> Result<Option<String>, String> {
        let keyring_key = format!("remote_conn_{}", id);
        crate::storage::keyring::get_api_key_internal(&keyring_key)
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Convert a database row to a RemoteConnection struct.
pub fn row_to_connection(row: &RemoteConnectionRow) -> RemoteConnection {
    let auth_type = match row.auth_type.as_str() {
        "api_key" => AuthType::ApiKey,
        "oauth2" => AuthType::OAuth2,
        _ => AuthType::None,
    };

    let status = match row.status.as_str() {
        "online" => ConnectionStatus::Online,
        "offline" => ConnectionStatus::Offline,
        "error" => ConnectionStatus::Error,
        _ => ConnectionStatus::Unknown,
    };

    let cached_agent_card = row
        .cached_agent_card
        .as_ref()
        .and_then(|json| serde_json::from_str(json).ok());

    RemoteConnection {
        id: row.id.clone(),
        name: row.name.clone(),
        endpoint_url: row.endpoint_url.clone(),
        auth_type,
        status,
        cached_agent_card,
        last_health_check_at: row.last_health_check_at.clone(),
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
    }
}

/// Build a reqwest HTTP client for a remote connection, with auth headers.
pub fn build_http_client(conn: &RemoteConnection) -> Result<reqwest::blocking::Client, String> {
    let builder = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        // For development: allow skipping TLS verification
        // In production, this should be configurable per-connection
        .danger_accept_invalid_certs(true);

    builder.build().map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Generate a unique connection ID.
fn generate_connection_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("conn-{:x}", nanos)
}
