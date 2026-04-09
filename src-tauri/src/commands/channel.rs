//! Tauri IPC commands for Channel operations.
//!
//! Provides CRUD operations for Channels, member management,
//! and basic message sending.

use crate::AppState;
use crate::workspace::channel::{
    self, Channel, ChannelInfo, ChannelMember, ChannelMessage, ChannelStore,
};
use tauri::{AppHandle, Emitter};

// ---------------------------------------------------------------------------
// Channel CRUD
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub member_agent_ids: Vec<String>,
}

/// Create a new Channel with the given name and Agent members.
#[tauri::command]
pub fn create_channel(
    state: tauri::State<'_, AppState>,
    request: CreateChannelRequest,
) -> Result<Channel, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    // Validate that all member agents exist
    for agent_id in &request.member_agent_ids {
        if manager.get_agent(agent_id).is_none() {
            return Err(format!("agent not found: {agent_id}"));
        }
    }

    let now = channel::now_iso();
    let channel_id = channel::generate_channel_id();

    let members: Vec<ChannelMember> = request
        .member_agent_ids
        .iter()
        .map(|aid| ChannelMember {
            agent_id: aid.clone(),
            role: "member".to_string(),
            joined_at: now.clone(),
        })
        .collect();

    let new_channel = Channel {
        id: channel_id,
        name: request.name,
        members,
        messages: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
    };

    let channels_dir = manager.channels_dir();
    let store = ChannelStore::new(&channels_dir);
    store.save(&new_channel).map_err(|e| format!("save failed: {e}"))?;

    log::info!(
        "[create_channel] channel_id={}, name={}",
        new_channel.id,
        new_channel.name
    );

    Ok(new_channel)
}

/// List all channels.
#[tauri::command]
pub fn list_channels(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ChannelInfo>, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let channels_dir = manager.channels_dir();
    let store = ChannelStore::new(&channels_dir);
    store.list().map_err(|e| format!("list failed: {e}"))
}

/// Get a single channel by ID (with full details including members).
#[tauri::command]
pub fn get_channel(
    state: tauri::State<'_, AppState>,
    channel_id: String,
) -> Result<Channel, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let channels_dir = manager.channels_dir();
    let store = ChannelStore::new(&channels_dir);
    store.load(&channel_id).map_err(|e| format!("load failed: {e}"))
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
}

/// Update a channel's settings (e.g., name).
#[tauri::command]
pub fn update_channel(
    state: tauri::State<'_, AppState>,
    channel_id: String,
    request: UpdateChannelRequest,
) -> Result<Channel, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let channels_dir = manager.channels_dir();
    let store = ChannelStore::new(&channels_dir);
    let mut channel = store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?;

    if let Some(name) = request.name {
        channel.name = name;
    }
    channel.updated_at = channel::now_iso();

    store.save(&channel).map_err(|e| format!("save failed: {e}"))?;

    log::info!("[update_channel] channel_id={}", channel_id);
    Ok(channel)
}

/// Delete a channel by ID.
#[tauri::command]
pub fn delete_channel(
    state: tauri::State<'_, AppState>,
    channel_id: String,
) -> Result<(), String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let channels_dir = manager.channels_dir();
    let store = ChannelStore::new(&channels_dir);
    store.delete(&channel_id).map_err(|e| format!("delete failed: {e}"))?;

    log::info!("[delete_channel] channel_id={}", channel_id);
    Ok(())
}

// ---------------------------------------------------------------------------
// Channel member management
// ---------------------------------------------------------------------------

/// Add an Agent member to a channel.
#[tauri::command]
pub fn add_channel_member(
    state: tauri::State<'_, AppState>,
    channel_id: String,
    agent_id: String,
) -> Result<Channel, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    // Verify agent exists
    if manager.get_agent(&agent_id).is_none() {
        return Err(format!("agent not found: {agent_id}"));
    }

    let channels_dir = manager.channels_dir();
    let store = ChannelStore::new(&channels_dir);
    let mut channel = store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?;

    // Check if already a member
    if channel.members.iter().any(|m| m.agent_id == agent_id) {
        return Err(format!("agent {agent_id} is already a member of channel {channel_id}"));
    }

    channel.members.push(ChannelMember {
        agent_id: agent_id.clone(),
        role: "member".to_string(),
        joined_at: channel::now_iso(),
    });
    channel.updated_at = channel::now_iso();

    store.save(&channel).map_err(|e| format!("save failed: {e}"))?;

    log::info!(
        "[add_channel_member] agent_id={} added to channel_id={}",
        agent_id,
        channel_id
    );
    Ok(channel)
}

/// Remove an Agent member from a channel.
#[tauri::command]
pub fn remove_channel_member(
    state: tauri::State<'_, AppState>,
    channel_id: String,
    agent_id: String,
) -> Result<Channel, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let channels_dir = manager.channels_dir();
    let store = ChannelStore::new(&channels_dir);
    let mut channel = store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?;

    let before = channel.members.len();
    channel.members.retain(|m| m.agent_id != agent_id);
    if channel.members.len() == before {
        return Err(format!("agent {agent_id} is not a member of channel {channel_id}"));
    }

    channel.updated_at = channel::now_iso();
    store.save(&channel).map_err(|e| format!("save failed: {e}"))?;

    log::info!(
        "[remove_channel_member] agent_id={} removed from channel_id={}",
        agent_id,
        channel_id
    );
    Ok(channel)
}

// ---------------------------------------------------------------------------
// Channel messaging
// ---------------------------------------------------------------------------

/// Send a message in a channel and trigger Agent response.
///
/// For now, routes the message to the first agent member for a single-agent reply.
/// Multi-agent @mention support is in feat-channel-multi-agent.
#[tauri::command]
pub async fn send_channel_message(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    channel_id: String,
    message: String,
) -> Result<Channel, String> {
    // 1. Load channel and add user message
    let channel = {
        let manager = state
            .agent_manager
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;

        let channels_dir = manager.channels_dir();
        let store = ChannelStore::new(&channels_dir);
        let mut channel = store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?;

        // Add user message
        let user_msg = ChannelMessage {
            id: crate::workspace::thread::generate_id(),
            channel_id: channel_id.clone(),
            sender_type: "user".to_string(),
            sender_id: "user".to_string(),
            content: message.clone(),
            timestamp: channel::now_iso(),
        };
        channel.messages.push(user_msg);
        channel.updated_at = channel::now_iso();

        // Get the first agent member as the default responder
        let responder_agent_id = channel
            .members
            .first()
            .map(|m| m.agent_id.clone())
            .ok_or_else(|| "channel has no agent members".to_string())?;

        // Get workspace path for runtime execution
        let workspace = manager
            .get_workspace(&responder_agent_id)
            .ok_or_else(|| format!("workspace not found for agent: {responder_agent_id}"))?;

        let workspace_path = workspace.base_path().to_string_lossy().to_string();

        // Get session info for the responder (create a session ID for channel context)
        let session_id = Some(crate::workspace::thread::generate_id());

        // Start runtime execution
        let receiver = {
            let registry = state
                .agent_runtime_registry
                .lock()
                .map_err(|e| e.to_string())?;
            let runtime = registry.get_runtime_instance("claude-code")?;

            let params = crate::runtime::ExecuteParams {
                message,
                session_id,
                workspace: Some(workspace_path),
                system_prompt: None,
                timeout_secs: 120,
            };

            runtime.execute(params)?
        };

        // Save channel with user message
        store.save(&channel).map_err(|e| format!("save failed: {e}"))?;

        // Spawn thread to forward streaming events to frontend
        let app_clone = app.clone();
        let channel_id_clone = channel_id.clone();
        let responder_id = responder_agent_id.clone();
        std::thread::spawn(move || {
            let mut full_response = String::new();
            let mut result_session_id: Option<String> = None;

            while let Ok(event) = receiver.recv() {
                if event.is_done {
                    result_session_id = event.session_id.clone();
                }
                if event.msg_type.as_deref() == Some("assistant") && !event.text.is_empty() {
                    full_response.push_str(&event.text);
                }
                // Forward event to frontend with channel context
                let _ = app_clone.emit(
                    "agent://channel-chunk",
                    serde_json::json!({
                        "channel_id": channel_id_clone,
                        "event": event,
                    }),
                );
                if event.is_done {
                    break;
                }
            }

            // Emit channel-response event for saving
            if !full_response.is_empty() {
                let _ = app_clone.emit("agent://channel-response", serde_json::json!({
                    "channel_id": channel_id_clone,
                    "agent_id": responder_id,
                    "content": full_response,
                    "session_id": result_session_id,
                }));
            }
        });

        channel
    };

    Ok(channel)
}

/// Save an agent response to a channel (called after streaming completes).
#[tauri::command]
pub fn save_channel_response(
    state: tauri::State<'_, AppState>,
    channel_id: String,
    agent_id: String,
    content: String,
) -> Result<Channel, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let channels_dir = manager.channels_dir();
    let store = ChannelStore::new(&channels_dir);
    let mut channel = store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?;

    let agent_msg = ChannelMessage {
        id: crate::workspace::thread::generate_id(),
        channel_id: channel_id.clone(),
        sender_type: "agent".to_string(),
        sender_id: agent_id,
        content,
        timestamp: channel::now_iso(),
    };
    channel.messages.push(agent_msg);
    channel.updated_at = channel::now_iso();

    store.save(&channel).map_err(|e| format!("save failed: {e}"))?;

    Ok(channel)
}
