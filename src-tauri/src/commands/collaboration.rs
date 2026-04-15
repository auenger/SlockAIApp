//! Tauri IPC commands for A2A multi-agent collaboration.
//!
//! Provides commands for:
//! - Task delegation (create, list, cancel, retry)
//! - Artifact management (list, get, search)
//! - Push notification configuration (register, unregister, list)
//!
//! ## Architecture
//!
//! Collaboration managers are stored as Tauri managed state via `CollaborationState`.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::runtime::a2a::{
    ArtifactRef, ArtifactStore, DelegationManager, DelegationRequest, DelegationStatus,
    PushEventType, PushNotification, PushCallbackConfig, PushNotificationManager,
};
use crate::runtime::a2a::types::ConnectionMode;

// ===========================================================================
// Collaboration State
// ===========================================================================

/// Managed state for collaboration features.
///
/// Holds the DelegationManager, ArtifactStore, and PushNotificationManager.
pub struct CollaborationState {
    pub delegation_manager: DelegationManager,
    pub artifact_store: ArtifactStore,
    pub push_manager: PushNotificationManager,
}

impl CollaborationState {
    /// Create a new CollaborationState with the given artifacts directory.
    pub fn new(artifacts_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            delegation_manager: DelegationManager::new(),
            artifact_store: ArtifactStore::new(artifacts_dir),
            push_manager: PushNotificationManager::new(),
        }
    }
}

// ===========================================================================
// Delegation Commands
// ===========================================================================

/// Request to create a delegation.
#[derive(Debug, Deserialize)]
pub struct CreateDelegationRequest {
    pub from_agent_id: String,
    pub to_agent_id: String,
    pub task_description: String,
    pub context_summary: String,
    #[serde(default)]
    pub parent_task_id: Option<String>,
    #[serde(default)]
    pub channel_id: Option<String>,
}

/// Delegation info returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct DelegationInfo {
    pub id: String,
    pub from_agent_id: String,
    pub to_agent_id: String,
    pub task_description: String,
    pub context_summary: String,
    pub parent_task_id: Option<String>,
    pub channel_id: Option<String>,
    pub status: String,
    pub target_connection_mode: Option<ConnectionMode>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<DelegationRequest> for DelegationInfo {
    fn from(d: DelegationRequest) -> Self {
        Self {
            id: d.id,
            from_agent_id: d.from_agent_id,
            to_agent_id: d.to_agent_id,
            task_description: d.task_description,
            context_summary: d.context_summary,
            parent_task_id: d.parent_task_id,
            channel_id: d.channel_id,
            status: d.status.as_str().to_string(),
            target_connection_mode: d.target_connection_mode,
            result: d.result,
            error: d.error,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

/// Create a new delegation.
#[tauri::command]
pub fn collaboration_delegate(
    state: State<'_, CollaborationState>,
    app: AppHandle,
    request: CreateDelegationRequest,
) -> Result<DelegationInfo, String> {
    // Resolve target connection mode from agent manager
    let target_connection_mode = {
        let app_state = app.state::<crate::AppState>();
        let manager = app_state
            .agent_manager
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;

        manager
            .get_agent(&request.to_agent_id)
            .map(|a| a.identity.connection_mode.clone())
    };

    let delegation = state.delegation_manager.create(
        &request.from_agent_id,
        &request.to_agent_id,
        &request.task_description,
        &request.context_summary,
        request.parent_task_id.as_deref(),
        request.channel_id.as_deref(),
        target_connection_mode,
    )?;

    // Emit event
    let _ = app.emit(
        "a2a://delegation-created",
        serde_json::json!({
            "delegation_id": delegation.id,
            "from_agent_id": delegation.from_agent_id,
            "to_agent_id": delegation.to_agent_id,
        }),
    );

    log::info!(
        "[collaboration_delegate] Created delegation {} ({} → {})",
        delegation.id,
        request.from_agent_id,
        request.to_agent_id
    );

    Ok(delegation.into())
}

/// List active delegations.
#[tauri::command]
pub fn collaboration_list_delegations(
    state: State<'_, CollaborationState>,
    agent_id: Option<String>,
    active_only: Option<bool>,
) -> Result<Vec<DelegationInfo>, String> {
    let delegations = if let Some(agent_id) = agent_id {
        let mut from = state.delegation_manager.list_by_from_agent(&agent_id)?;
        let to = state.delegation_manager.list_by_to_agent(&agent_id)?;
        from.extend(to);
        from.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        from.dedup_by(|a, b| a.id == b.id);
        from
    } else if active_only.unwrap_or(false) {
        state.delegation_manager.list_active()?
    } else {
        state.delegation_manager.list_all()?
    };

    Ok(delegations.into_iter().map(DelegationInfo::from).collect())
}

/// Cancel a delegation.
#[tauri::command]
pub fn collaboration_cancel_delegation(
    state: State<'_, CollaborationState>,
    app: AppHandle,
    delegation_id: String,
) -> Result<DelegationInfo, String> {
    let delegation = state.delegation_manager.cancel(&delegation_id)?;

    let _ = app.emit(
        "a2a://delegation-cancelled",
        serde_json::json!({
            "delegation_id": delegation_id,
        }),
    );

    Ok(delegation.into())
}

/// Retry a failed delegation.
#[tauri::command]
pub fn collaboration_retry_delegation(
    state: State<'_, CollaborationState>,
    app: AppHandle,
    delegation_id: String,
) -> Result<DelegationInfo, String> {
    let existing = state
        .delegation_manager
        .get(&delegation_id)?
        .ok_or_else(|| format!("Delegation not found: {}", delegation_id))?;

    if existing.status != DelegationStatus::Failed && existing.status != DelegationStatus::TimedOut {
        return Err("Only failed or timed-out delegations can be retried".to_string());
    }

    // Create a new delegation with the same parameters
    let new_delegation = state.delegation_manager.create(
        &existing.from_agent_id,
        &existing.to_agent_id,
        &existing.task_description,
        &existing.context_summary,
        existing.parent_task_id.as_deref(),
        existing.channel_id.as_deref(),
        existing.target_connection_mode,
    )?;

    let _ = app.emit(
        "a2a://delegation-retried",
        serde_json::json!({
            "old_delegation_id": delegation_id,
            "new_delegation_id": new_delegation.id,
        }),
    );

    Ok(new_delegation.into())
}

// ===========================================================================
// Artifact Commands
// ===========================================================================

/// Artifact info returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct ArtifactInfo {
    pub id: String,
    pub producer_agent_id: String,
    pub name: String,
    pub file_path: String,
    pub content_hash: Option<String>,
    pub mime_type: Option<String>,
    pub created_at: String,
    pub task_id: Option<String>,
    pub description: Option<String>,
    pub size: u64,
}

impl From<ArtifactRef> for ArtifactInfo {
    fn from(a: ArtifactRef) -> Self {
        Self {
            id: a.id,
            producer_agent_id: a.producer_agent_id,
            name: a.name,
            file_path: a.file_path,
            content_hash: a.content_hash,
            mime_type: a.mime_type,
            created_at: a.created_at,
            task_id: a.task_id,
            description: a.description,
            size: a.size,
        }
    }
}

/// List artifacts with optional filters.
#[tauri::command]
pub fn collaboration_list_artifacts(
    state: State<'_, CollaborationState>,
    agent_id: Option<String>,
    task_id: Option<String>,
) -> Result<Vec<ArtifactInfo>, String> {
    let artifacts = if let Some(task_id) = task_id {
        state.artifact_store.list_by_task(&task_id)?
    } else if let Some(agent_id) = agent_id {
        state.artifact_store.list_by_producer(&agent_id)?
    } else {
        state.artifact_store.list_all()?
    };

    Ok(artifacts.into_iter().map(ArtifactInfo::from).collect())
}

/// Get artifact content.
#[tauri::command]
pub fn collaboration_get_artifact(
    state: State<'_, CollaborationState>,
    artifact_id: String,
    consumer_agent_id: Option<String>,
) -> Result<ArtifactContentResult, String> {
    let content = state
        .artifact_store
        .get_content(&artifact_id)?
        .ok_or_else(|| format!("Artifact not found: {}", artifact_id))?;

    let meta = state
        .artifact_store
        .get(&artifact_id)?
        .ok_or_else(|| format!("Artifact metadata not found: {}", artifact_id))?;

    // Record consumption if consumer is specified
    if let Some(consumer_id) = consumer_agent_id {
        state
            .artifact_store
            .record_consumption(&artifact_id, &consumer_id)?;
    }

    Ok(ArtifactContentResult {
        artifact: meta.into(),
        content,
    })
}

/// Result of getting artifact content.
#[derive(Debug, Serialize)]
pub struct ArtifactContentResult {
    pub artifact: ArtifactInfo,
    pub content: String,
}

/// Search artifacts by name.
#[tauri::command]
pub fn collaboration_search_artifacts(
    state: State<'_, CollaborationState>,
    query: String,
) -> Result<Vec<ArtifactInfo>, String> {
    let artifacts = state.artifact_store.search(&query)?;
    Ok(artifacts.into_iter().map(ArtifactInfo::from).collect())
}

/// Register a new artifact.
#[tauri::command]
pub fn collaboration_register_artifact(
    state: State<'_, CollaborationState>,
    producer_agent_id: String,
    name: String,
    file_path: String,
    mime_type: Option<String>,
    task_id: Option<String>,
    description: Option<String>,
) -> Result<ArtifactInfo, String> {
    let artifact = state.artifact_store.register(
        &producer_agent_id,
        &name,
        &file_path,
        mime_type.as_deref(),
        task_id.as_deref(),
        description.as_deref(),
    )?;

    Ok(artifact.into())
}

// ===========================================================================
// Push Notification Commands
// ===========================================================================

/// Register a push notification endpoint.
#[tauri::command]
pub fn collaboration_register_push_url(
    state: State<'_, CollaborationState>,
    url: String,
    token: Option<String>,
    hmac_secret: Option<String>,
    events: Option<Vec<String>>,
) -> Result<PushNotificationConfigInfo, String> {
    let id = crate::runtime::a2a::push::generate_push_config_id();

    let config = PushCallbackConfig {
        id,
        url,
        token,
        hmac_secret,
        events: events.unwrap_or_default(),
        active: true,
    };

    state.push_manager.register_config(config.clone())?;

    Ok(PushNotificationConfigInfo {
        id: config.id,
        url: config.url,
        has_token: config.token.is_some(),
        has_hmac_secret: config.hmac_secret.is_some(),
        events: config.events,
        active: config.active,
    })
}

/// List push notification configs.
#[tauri::command]
pub fn collaboration_list_push_configs(
    state: State<'_, CollaborationState>,
) -> Result<Vec<PushNotificationConfigInfo>, String> {
    let configs = state.push_manager.list_configs()?;
    Ok(configs
        .into_iter()
        .map(|c| PushNotificationConfigInfo {
            id: c.id,
            url: c.url,
            has_token: c.token.is_some(),
            has_hmac_secret: c.hmac_secret.is_some(),
            events: c.events,
            active: c.active,
        })
        .collect())
}

/// Unregister a push notification config.
#[tauri::command]
pub fn collaboration_unregister_push_url(
    state: State<'_, CollaborationState>,
    config_id: String,
) -> Result<bool, String> {
    state.push_manager.unregister_config(&config_id)
}

/// Manually process a push notification (for testing / webhook relay).
#[tauri::command]
pub fn collaboration_process_push_event(
    state: State<'_, CollaborationState>,
    app: AppHandle,
    notification: PushNotification,
) -> Result<bool, String> {
    state.push_manager.process_event(&notification, &app)
}

/// Push notification config info (safe for frontend, no secrets).
#[derive(Debug, Clone, Serialize)]
pub struct PushNotificationConfigInfo {
    pub id: String,
    pub url: String,
    pub has_token: bool,
    pub has_hmac_secret: bool,
    pub events: Vec<String>,
    pub active: bool,
}
