//! Tauri IPC commands for Channel operations.
//!
//! Provides CRUD operations for Channels, member management,
//! message sending with multi-Agent @mention support,
//! and context-orchestrated execution.
//!
//! ## Dual-Write Strategy
//!
//! Activity logs are written to both JSONL and SQLite.
//! Channel metadata is tracked in SQLite for fast listing.

use crate::workspace::channel::{
    self, Channel, ChannelInfo, ChannelMember, ChannelMessage, ChannelStore,
};
use crate::workspace::mention;
use crate::context::a2a_trigger::{self, TriggerContext};
use crate::storage::activity::{ActivityStore, ActivityType, create_entry};
use crate::storage::db_helpers;
use tauri::{AppHandle, Emitter, Manager};

// ---------------------------------------------------------------------------
// Helper: resolve user display name from USER.md
// ---------------------------------------------------------------------------

/// Extract the user's display name from USER.md.
/// Looks for `- **Name**: value` pattern. Falls back to "User".
fn resolve_user_name(workspace_root: &std::path::Path) -> String {
    let user_md = workspace_root.join("USER.md");
    if let Ok(content) = std::fs::read_to_string(&user_md) {
        // Match patterns like: - **Name**: Ryan  or  - **Name**: Ryan_
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- **Name**") {
                if let Some(colon_pos) = trimmed.find(':') {
                    let value = trimmed[colon_pos + 1..].trim();
                    // Strip markdown italic markers and placeholder
                    let cleaned = value
                        .trim_start_matches('_')
                        .trim_end_matches('_')
                        .trim();
                    if !cleaned.is_empty()
                        && !cleaned.starts_with('(')
                        && cleaned != "your name"
                    {
                        return cleaned.to_string();
                    }
                }
            }
        }
    }
    "User".to_string()
}

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
    state: tauri::State<'_, crate::AppState>,
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
        summary: None,
        summary_up_to: None,
        summary_updated_at: None,
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

    // Insert channel metadata into SQLite
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        let channel_row = db_helpers::ChannelRow {
            id: new_channel.id.clone(),
            name: new_channel.name.clone(),
            messages_jsonl_path: None,
            created_at: new_channel.created_at.clone(),
            updated_at: new_channel.updated_at.clone(),
        };
        if let Err(e) = db_helpers::insert_channel(&db_conn, &channel_row) {
            log::warn!("[create_channel] Failed to insert channel into SQLite: {}", e);
        }

        // Ensure agents exist in SQLite (upsert) before inserting members
        // to satisfy the foreign key constraint on channel_members.agent_id
        for member in &new_channel.members {
            if let Some(agent) = manager.get_agent(&member.agent_id) {
                let agent_row = db_helpers::AgentRow {
                    id: agent.identity.agent_id.clone(),
                    name: agent.identity.name.clone(),
                    emoji: agent.identity.emoji.clone(),
                    avatar_path: None,
                    enabled: true,
                    runtime_type: agent.identity.runtime_type.runtime_id().to_string(),
                    description: agent.identity.vibe.clone(),
                    created_at: new_channel.created_at.clone(),
                    updated_at: new_channel.updated_at.clone(),
                };
                if let Err(e) = db_helpers::insert_agent(&db_conn, &agent_row) {
                    log::warn!("[create_channel] Failed to upsert agent into SQLite: {}", e);
                }
            }
        }

        // Insert channel members into SQLite
        for member in &new_channel.members {
            let member_row = db_helpers::ChannelMemberRow {
                channel_id: new_channel.id.clone(),
                agent_id: member.agent_id.clone(),
                role: member.role.clone(),
                joined_at: member.joined_at.clone(),
            };
            if let Err(e) = db_helpers::insert_channel_member(&db_conn, &member_row) {
                log::warn!("[create_channel] Failed to insert channel member into SQLite: {}", e);
            }
        }

        // Log activity (dual-write: JSONL + SQLite)
        let activity_store = ActivityStore::new(manager.workspace_root());
        let member_ids: Vec<&str> = new_channel.members.iter().map(|m| m.agent_id.as_str()).collect();
        let entry = create_entry(
            ActivityType::ChannelCreated,
            None,
            format!("Channel \"{}\" created with {} members", new_channel.name, new_channel.members.len()),
            serde_json::json!({
                "channel_id": new_channel.id,
                "channel_name": new_channel.name,
                "members": member_ids,
            }),
        );
        if let Err(e) = activity_store.append(&entry) {
            log::warn!("[create_channel] Failed to log activity to JSONL: {}", e);
        }

        let activity_type_str = serde_json::to_string(&ActivityType::ChannelCreated)
            .unwrap_or_else(|_| "\"system\"".to_string())
            .trim_matches('"')
            .to_string();
        let db_row = db_helpers::ActivityLogRow {
            id: entry.id,
            timestamp: entry.timestamp,
            activity_type: activity_type_str,
            agent_id: entry.agent_id,
            workspace_id: entry.workspace_id,
            summary: entry.summary,
            details_json: serde_json::to_string(&entry.details).unwrap_or_else(|_| "{}".to_string()),
        };
        if let Err(e) = db_helpers::insert_activity(&db_conn, &db_row) {
            log::warn!("[create_channel] Failed to log activity to SQLite: {}", e);
        }
    }

    Ok(new_channel)
}

/// List all channels.
#[tauri::command]
pub fn list_channels(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<ChannelInfo>, String> {
    // Try SQLite first for fast listing
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        if let Ok(rows) = db_helpers::list_channels(&db_conn) {
            if !rows.is_empty() {
                let channel_infos: Vec<ChannelInfo> = rows.into_iter().map(|r| {
                    ChannelInfo {
                        id: r.id,
                        name: r.name,
                        member_count: 0, // Would need join query, will be filled from file fallback
                        unread_count: 0,
                        preview: String::new(),
                        message_count: 0,
                        created_at: r.created_at,
                        updated_at: r.updated_at,
                    }
                }).collect();
                // If we have SQLite data, supplement with member counts from file
                let manager = state
                    .agent_manager
                    .lock()
                    .map_err(|e| format!("lock error: {e}"))?;
                let channels_dir = manager.channels_dir();
                let store = ChannelStore::new(&channels_dir);
                let file_channels = store.list().unwrap_or_default();
                let file_map: std::collections::HashMap<String, ChannelInfo> = file_channels
                    .into_iter()
                    .map(|c| (c.id.clone(), c))
                    .collect();
                let merged: Vec<ChannelInfo> = channel_infos.into_iter().map(|mut ci| {
                    if let Some(fc) = file_map.get(&ci.id) {
                        ci.member_count = fc.member_count;
                        ci.message_count = fc.message_count;
                        ci.preview = fc.preview.clone();
                    }
                    ci
                }).collect();
                return Ok(merged);
            }
        }
    }

    // Fallback to file-based listing
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
    state: tauri::State<'_, crate::AppState>,
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
    state: tauri::State<'_, crate::AppState>,
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

    // Update channel metadata in SQLite
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        let channel_row = db_helpers::ChannelRow {
            id: channel.id.clone(),
            name: channel.name.clone(),
            messages_jsonl_path: None,
            created_at: channel.created_at.clone(),
            updated_at: channel.updated_at.clone(),
        };
        if let Err(e) = db_helpers::insert_channel(&db_conn, &channel_row) {
            log::warn!("[update_channel] Failed to update channel in SQLite: {}", e);
        }

        // Log activity (dual-write: JSONL + SQLite)
        let activity_store = ActivityStore::new(manager.workspace_root());
        let entry = create_entry(
            ActivityType::ChannelUpdated,
            None,
            format!("Channel \"{}\" updated", channel.name),
            serde_json::json!({ "channel_id": channel_id, "channel_name": channel.name }),
        );
        if let Err(e) = activity_store.append(&entry) {
            log::warn!("[update_channel] Failed to log activity to JSONL: {}", e);
        }

        let activity_type_str = serde_json::to_string(&ActivityType::ChannelUpdated)
            .unwrap_or_else(|_| "\"system\"".to_string())
            .trim_matches('"')
            .to_string();
        let db_row = db_helpers::ActivityLogRow {
            id: entry.id,
            timestamp: entry.timestamp,
            activity_type: activity_type_str,
            agent_id: entry.agent_id,
            workspace_id: entry.workspace_id,
            summary: entry.summary,
            details_json: serde_json::to_string(&entry.details).unwrap_or_else(|_| "{}".to_string()),
        };
        if let Err(e) = db_helpers::insert_activity(&db_conn, &db_row) {
            log::warn!("[update_channel] Failed to log activity to SQLite: {}", e);
        }
    }

    Ok(channel)
}

/// Delete a channel by ID.
#[tauri::command]
pub fn delete_channel(
    state: tauri::State<'_, crate::AppState>,
    channel_id: String,
) -> Result<(), String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let channels_dir = manager.channels_dir();
    let store = ChannelStore::new(&channels_dir);
    store.delete(&channel_id).map_err(|e| format!("delete failed: {e}"))?;

    // Delete from SQLite
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        if let Err(e) = db_helpers::delete_channel(&db_conn, &channel_id) {
            log::warn!("[delete_channel] Failed to delete channel from SQLite: {}", e);
        }
    }

    log::info!("[delete_channel] channel_id={}", channel_id);

    // Log activity (dual-write: JSONL + SQLite)
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        let activity_store = ActivityStore::new(manager.workspace_root());
        let entry = create_entry(
            ActivityType::ChannelDeleted,
            None,
            format!("Channel {} deleted", channel_id),
            serde_json::json!({ "channel_id": channel_id }),
        );
        if let Err(e) = activity_store.append(&entry) {
            log::warn!("[delete_channel] Failed to log activity to JSONL: {}", e);
        }

        let activity_type_str = serde_json::to_string(&ActivityType::ChannelDeleted)
            .unwrap_or_else(|_| "\"system\"".to_string())
            .trim_matches('"')
            .to_string();
        let db_row = db_helpers::ActivityLogRow {
            id: entry.id,
            timestamp: entry.timestamp,
            activity_type: activity_type_str,
            agent_id: entry.agent_id,
            workspace_id: entry.workspace_id,
            summary: entry.summary,
            details_json: serde_json::to_string(&entry.details).unwrap_or_else(|_| "{}".to_string()),
        };
        if let Err(e) = db_helpers::insert_activity(&db_conn, &db_row) {
            log::warn!("[delete_channel] Failed to log activity to SQLite: {}", e);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Channel member management
// ---------------------------------------------------------------------------

/// Add an Agent member to a channel.
#[tauri::command]
pub fn add_channel_member(
    state: tauri::State<'_, crate::AppState>,
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
    state: tauri::State<'_, crate::AppState>,
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
// Channel messaging with multi-Agent @mention support
// ---------------------------------------------------------------------------

/// Maximum number of recent messages to include as Channel context.
const CHANNEL_CONTEXT_HISTORY_LIMIT: usize = 20;

// ---------------------------------------------------------------------------
// Single-agent execution helper
// ---------------------------------------------------------------------------

/// Execute a single agent in a channel context, streaming events to the frontend.
///
/// Returns the full response text on success, or an error string on failure.
/// This function is synchronous — all async work happens inside the spawned CLI thread.
fn execute_single_agent_inner(
    app: &AppHandle,
    state: &crate::AppState,
    channel_id: &str,
    agent_id: &str,
    message: &str,
    agent_idx: usize,
    total_agents: usize,
    workspace_root: &std::path::Path,
    is_a2a: bool,
    triggered_by: Option<&str>,
    a2a_depth: u32,
    user_name: Option<&str>,
) -> Result<String, String> {
    let channel_id = channel_id.to_string();
    let agent_id = agent_id.to_string();
    let message = message.to_string();
    let workspace_root = workspace_root.to_path_buf();

    // Build context for this agent
    let (system_prompt, workspace_path, recent_count, runtime_id, runtime_name) = {
        let manager = state
            .agent_manager
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;

        // Load latest channel state for context
        let channels_dir = manager.channels_dir();
        let store = ChannelStore::new(&channels_dir);
        let latest_channel = store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?;

        // Collect all agents that are members of this channel
        let channel_agents: Vec<crate::workspace::manager::Agent> = latest_channel
            .members
            .iter()
            .filter_map(|m| manager.get_agent(&m.agent_id))
            .cloned()
            .collect();

        // Build Zone Agent Protocol (L2) from channel members + user name
        // Prefer the name passed from frontend (Profile setting), fallback to USER.md
        let effective_user_name: String = user_name
            .filter(|n| !n.is_empty() && *n != "User")
            .map(|s| s.to_string())
            .unwrap_or_else(|| resolve_user_name(&workspace_root));
        let zone_protocol = crate::context::zone_protocol::ChannelZoneProtocol::from_channel(
            &latest_channel,
            &effective_user_name,
            &channel_agents,
        );

        // Build agent context via ContextBuilder, with Zone Protocol injected
        let builder = crate::context::ContextBuilder::new(&workspace_root)
            .with_zone_protocol(zone_protocol);
        let mut context_prefix = builder
            .build_context_prefix(&agent_id)
            .unwrap_or_default();

        // --- Sliding window + summary context ---
        let summary_up_to_idx = match &latest_channel.summary_up_to {
            Some(sid) => latest_channel
                .messages
                .iter()
                .position(|m| m.id == *sid)
                .map(|i| i + 1)
                .unwrap_or(0),
            None => 0,
        };

        let recent_start = if summary_up_to_idx > 0 {
            summary_up_to_idx
        } else {
            latest_channel
                .messages
                .len()
                .saturating_sub(CHANNEL_CONTEXT_HISTORY_LIMIT)
        };

        let recent: Vec<&ChannelMessage> = latest_channel
            .messages
            .iter()
            .skip(recent_start)
            .collect();

        // Include summary if available
        if let Some(ref summary) = latest_channel.summary {
            context_prefix.push_str("\n\n# Earlier Conversation Summary\n\n");
            context_prefix.push_str(summary);
            context_prefix.push_str("\n\n");
        }

        // Append recent conversation history
        if !recent.is_empty() {
            context_prefix.push_str("# Channel Recent Messages\n\n");
            context_prefix.push_str(&format!(
                "You are in a channel named \"{}\". Here are the recent messages:\n\n",
                latest_channel.name
            ));
            for msg in &recent {
                let sender = if msg.sender_type == "user" {
                    resolve_user_name(&workspace_root)
                } else {
                    manager
                        .get_agent(&msg.sender_id)
                        .map(|a| a.identity.name.clone())
                        .unwrap_or_else(|| msg.sender_id.clone())
                };
                context_prefix.push_str(&format!("[{}]: {}\n", sender, msg.content));
            }
            context_prefix.push_str("\n---\n\n");
        }

        // Get workspace path for this agent
        let workspace = manager
            .get_workspace(&agent_id)
            .ok_or_else(|| format!("workspace not found for agent: {agent_id}"))?;
        let ws_path = workspace.base_path().to_string_lossy().to_string();

        // Resolve the agent's runtime type
        let agent = manager
            .get_agent(&agent_id)
            .ok_or_else(|| format!("agent not found: {agent_id}"))?;
        let rt_id = agent.identity.runtime_type.runtime_id().to_string();
        let rt_name = agent.identity.runtime_type.display_name().to_string();

        (Some(context_prefix), ws_path, recent.len(), rt_id, rt_name)
    };

    log::info!(
        "[execute_single_agent] agent {}/{}, agent_id={}, runtime={}, context: {} recent messages, a2a={}",
        agent_idx + 1,
        total_agents,
        agent_id,
        runtime_id,
        recent_count,
        is_a2a,
    );

    // Emit agent-start event so frontend knows which agent is responding
    let _ = app.emit(
        "agent://channel-agent-start",
        serde_json::json!({
            "channel_id": channel_id,
            "agent_id": agent_id,
            "agent_index": agent_idx,
            "total_agents": total_agents,
            "runtime_id": runtime_id,
            "runtime_name": runtime_name,
            "is_a2a": is_a2a,
            "triggered_by": triggered_by,
            "a2a_depth": a2a_depth,
        }),
    );

    // If this is an A2A trigger, emit the A2A-specific event too
    if is_a2a {
        let _ = app.emit(
            "agent://channel-a2a-start",
            serde_json::json!({
                "channel_id": channel_id,
                "agent_id": agent_id,
                "triggered_by": triggered_by.unwrap_or("unknown"),
                "depth": a2a_depth,
            }),
        );
    }

    // Start runtime execution for this agent
    let receiver = {
        let registry = state
            .agent_runtime_registry
            .lock()
            .map_err(|e| e.to_string())?;

        let runtime = registry.get_runtime_instance(&runtime_id)?;

        // Health check: verify the runtime is available before executing
        if !runtime.is_ready() {
            let info = registry.get_runtime(&runtime_id);
            let install_hint = info
                .map(|i| i.install_hint.clone())
                .unwrap_or_default();
            let error_msg = format!(
                "{} runtime is not available for agent {}. Please install: {}",
                runtime.name(),
                agent_id,
                install_hint,
            );
            let _ = app.emit("runtime://unavailable", serde_json::json!({
                "channel_id": channel_id,
                "agent_id": agent_id,
                "runtime_id": runtime_id,
                "runtime_name": runtime.name(),
                "install_hint": install_hint,
                "error": error_msg,
            }));
            return Err(error_msg);
        }

        let params = crate::runtime::ExecuteParams {
            agent_id: agent_id.clone(),
            message: message.clone(),
            session_id: None,
            workspace: Some(workspace_path),
            system_prompt,
            timeout_secs: 120,
        };

        runtime.execute(params)?
    };

    // Spawn thread to forward streaming events and collect response
    let app_clone = app.clone();
    let channel_id_clone = channel_id.clone();
    let agent_id_clone = agent_id.clone();
    let agent_manager_ptr = {
        state
            .agent_manager
            .lock()
            .map_err(|e| format!("lock error: {e}"))?
            .workspace_root()
            .to_path_buf()
    };

    let (tx_done, rx_done) = std::sync::mpsc::channel::<Result<String, String>>();

    std::thread::spawn(move || {
        let mut full_response = String::new();
        let mut result_session_id: Option<String> = None;
        let mut had_error = false;
        let mut first_chunk_received = false;
        let mut collected_blocks: Vec<serde_json::Value> = Vec::new();

        log::info!(
            "[response-thread] agent={}, channel={}: started collecting response",
            agent_id_clone,
            channel_id_clone
        );

        while let Ok(event) = receiver.recv() {
            if !first_chunk_received {
                first_chunk_received = true;
                log::info!(
                    "[response-thread] agent={}, channel={}: first chunk received, type={:?}, is_done={}",
                    agent_id_clone,
                    channel_id_clone,
                    event.msg_type,
                    event.is_done
                );
            }

            if event.is_done {
                result_session_id = event.session_id.clone();
            }
            if event.msg_type.as_deref() == Some("assistant") && !event.text.is_empty() {
                full_response.push_str(&event.text);
            }

            // Collect content_blocks from assistant and user events
            if let Some(ref blocks_val) = event.content_blocks {
                if let Some(arr) = blocks_val.as_array() {
                    for block in arr {
                        collected_blocks.push(block.clone());
                    }
                }
            }

            // Forward event to frontend with agent_id context
            let _ = app_clone.emit(
                "agent://channel-chunk",
                serde_json::json!({
                    "channel_id": channel_id_clone,
                    "agent_id": agent_id_clone,
                    "agent_index": agent_idx,
                    "total_agents": total_agents,
                    "event": event,
                }),
            );

            if event.is_done {
                log::info!(
                    "[response-thread] agent={}, channel={}: is_done received, collected {} chars, {} content_blocks, session_id={:?}, error={:?}",
                    agent_id_clone,
                    channel_id_clone,
                    full_response.len(),
                    collected_blocks.len(),
                    result_session_id,
                    event.error
                );
                if event.error.is_some() {
                    had_error = true;
                    log::warn!(
                        "[execute_single_agent] agent {} had error: {:?}",
                        agent_id_clone,
                        event.error
                    );
                }
                break;
            }
        }

        // Save agent response to channel
        if !full_response.is_empty() && !had_error {
            let channels_dir = agent_manager_ptr.join("channels");
            let store = ChannelStore::new(&channels_dir);
            match store.load(&channel_id_clone) {
                Ok(mut ch) => {
                    let content_blocks = if collected_blocks.is_empty() {
                        None
                    } else {
                        Some(collected_blocks.clone())
                    };
                    let agent_msg = ChannelMessage {
                        id: crate::workspace::thread::generate_id(),
                        channel_id: channel_id_clone.clone(),
                        sender_type: "agent".to_string(),
                        sender_id: agent_id_clone.clone(),
                        content: full_response.clone(),
                        content_blocks,
                        timestamp: channel::now_iso(),
                    };
                    ch.messages.push(agent_msg);
                    ch.updated_at = channel::now_iso();
                    match store.save(&ch) {
                        Ok(()) => {
                            log::info!(
                                "[response-thread] agent={}, channel={}: response saved to channel store ({} total messages)",
                                agent_id_clone,
                                channel_id_clone,
                                ch.messages.len()
                            );
                        }
                        Err(e) => {
                            log::error!(
                                "[response-thread] agent={}, channel={}: FAILED to save response to channel store: {}",
                                agent_id_clone,
                                channel_id_clone,
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    log::error!(
                        "[response-thread] agent={}, channel={}: FAILED to load channel for saving response: {}",
                        agent_id_clone,
                        channel_id_clone,
                        e
                    );
                }
            }

            // Emit response event for frontend (with content_blocks)
            let content_blocks_val = if collected_blocks.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::Array(collected_blocks)
            };
            let _ = app_clone.emit(
                "agent://channel-response",
                serde_json::json!({
                    "channel_id": channel_id_clone,
                    "agent_id": agent_id_clone,
                    "content": full_response.clone(),
                    "content_blocks": content_blocks_val,
                    "session_id": result_session_id,
                }),
            );
            log::info!(
                "[response-thread] agent={}, channel={}: emitted channel-response event",
                agent_id_clone,
                channel_id_clone
            );
        } else {
            log::warn!(
                "[response-thread] agent={}, channel={}: skipping save (empty={}, error={})",
                agent_id_clone,
                channel_id_clone,
                full_response.is_empty(),
                had_error
            );
        }

        // Signal completion with the response text
        let result = if had_error {
            Err("agent execution had error".to_string())
        } else {
            Ok(full_response)
        };
        let _ = tx_done.send(result);
    });

    // Wait for the agent to complete
    match rx_done.recv_timeout(std::time::Duration::from_secs(300)) {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(e)) => {
            log::warn!("[execute_single_agent] agent {} failed: {}", agent_id, e);
            Err(e)
        }
        Err(_) => {
            log::warn!("[execute_single_agent] agent {} timed out", agent_id);
            Err("agent execution timed out".to_string())
        }
    }
}

/// Send a message in a channel, parse @mentions, and trigger multi-Agent responses.
///
/// Flow:
/// 1. Parse @mentions from the user message
/// 2. Resolve to agent IDs (fallback to first member if no mentions)
/// 3. For each mentioned agent (serial execution):
///    a. Build agent context (SOUL.md + IDENTITY.md + MEMORY.md)
///    b. Build channel context (recent N messages as conversation history)
///    c. Execute via runtime with assembled system prompt
///    d. Stream events to frontend with agent_id identifier
/// 4. After each agent responds, check for A2A triggers (@agent mentions
///    in the response) and recursively execute those agents (with depth
///    limit and deduplication to prevent runaway chains).
/// 5. Each agent's response is saved independently

/// Response from `send_channel_message`: the updated channel plus metadata
/// about how many agents were triggered (so the frontend knows whether to
/// expect streaming events).
#[derive(Debug, serde::Serialize)]
pub struct SendChannelMessageResponse {
    /// The channel (with the user message already persisted).
    pub channel: Channel,
    /// Number of agents triggered by @mentions (0 = no agents, clean up streaming state).
    pub agents_triggered: usize,
}

#[tauri::command]
pub async fn send_channel_message(
    app: AppHandle,
    state: tauri::State<'_, crate::AppState>,
    channel_id: String,
    message: String,
    user_name: Option<String>,
) -> Result<SendChannelMessageResponse, String> {
    // ---- Phase 1: Load channel, add user message, parse mentions ----
    let (_channel, target_agents, workspace_root, channel_members) = {
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
            content_blocks: None,
            timestamp: channel::now_iso(),
        };
        channel.messages.push(user_msg);
        channel.updated_at = channel::now_iso();

        // Parse @mentions
        let mention_result = mention::parse_mentions(&message, &channel.members);
        let target_agents = mention::resolve_agents(&mention_result.mentions, &channel.members);

        // Save channel with user message (always, regardless of mentions)
        store.save(&channel).map_err(|e| format!("save failed: {e}"))?;

        if target_agents.is_empty() {
            // No @mentions — save user message but don't trigger any agent
            log::info!(
                "[send_channel_message] channel_id={}, no @mentions, skipping agent execution",
                channel_id
            );
            return Ok(SendChannelMessageResponse {
                channel,
                agents_triggered: 0,
            });
        }

        log::info!(
            "[send_channel_message] channel_id={}, parsed {} mentions, targets: {:?}",
            channel_id,
            mention_result.mentions.len(),
            target_agents
        );

        let workspace_root = manager.workspace_root().to_path_buf();
        let channel_members = channel.members.clone();

        (channel, target_agents, workspace_root, channel_members)
    };

    // ---- Phase 2: Execute agents in background, return immediately ----
    //
    // CRITICAL: The IPC must return immediately so the frontend's event listeners
    // can receive streaming events (agent-start, channel-chunk, channel-response).
    // If we block the IPC until all agents complete, the WebView event loop is
    // stalled and events won't be delivered until after the invoke resolves.
    //
    // We spawn a background thread that runs the agent execution loop and emits
    // events. The frontend tracks progress via these events.

    /// A pending agent execution task.
    struct PendingAgentTask {
        agent_id: String,
        message: String,
        is_a2a: bool,
        triggered_by: Option<String>,
        depth: u32,
    }

    let total_agents = target_agents.len();
    let max_depth = a2a_trigger::DEFAULT_MAX_DEPTH;
    let mut triggered_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut task_queue: std::collections::VecDeque<PendingAgentTask> = std::collections::VecDeque::new();
    for agent_id in &target_agents {
        triggered_set.insert(agent_id.clone());
        task_queue.push_back(PendingAgentTask {
            agent_id: agent_id.clone(),
            message: message.clone(),
            is_a2a: false,
            triggered_by: None,
            depth: 0,
        });
    }

    // Spawn background thread — accesses AppState through AppHandle
    let bg_app = app.clone();
    let bg_channel_id = channel_id.clone();
    let bg_workspace_root = workspace_root.clone();
    let bg_channel_members = channel_members.clone();

    std::thread::spawn(move || {
        // Access AppState through the AppHandle (valid for the thread's lifetime)
        let app_state = bg_app.state::<crate::AppState>();
        let state: &crate::AppState = app_state.inner();

        let mut agent_idx = 0usize;
        let mut triggered = triggered_set;

        while let Some(task) = task_queue.pop_front() {
            let response_result = execute_single_agent_inner(
                &bg_app,
                state,
                &bg_channel_id,
                &task.agent_id,
                &task.message,
                agent_idx,
                total_agents + task_queue.len(),
                &bg_workspace_root,
                task.is_a2a,
                task.triggered_by.as_deref(),
                task.depth,
                user_name.as_deref(),
            );

            agent_idx += 1;

            // If the agent responded successfully, check for A2A triggers
            if let Ok(response) = response_result {
                let current_depth = task.depth;

                if current_depth < max_depth {
                    let next_triggers = a2a_trigger::extract_valid_triggers(
                        &response,
                        &bg_channel_members,
                        &TriggerContext {
                            depth: current_depth,
                            max_depth,
                            triggered_agents: triggered.clone(),
                        },
                    );

                    for triggered_agent_id in next_triggers {
                        let triggered_by = task.agent_id.clone();
                        log::info!(
                            "[send_channel_message:bg] A2A trigger: {} → {} (depth={})",
                            triggered_by,
                            triggered_agent_id,
                            current_depth + 1,
                        );

                        triggered.insert(triggered_agent_id.clone());

                        let a2a_message = format!(
                            "[A2A Trigger] {} mentioned you in their response. Here is what they said:\n\n{}\n\nPlease respond to the relevant request above.",
                            triggered_by, response
                        );

                        task_queue.push_back(PendingAgentTask {
                            agent_id: triggered_agent_id,
                            message: a2a_message,
                            is_a2a: true,
                            triggered_by: Some(triggered_by),
                            depth: current_depth + 1,
                        });
                    }
                } else {
                    let mentioned = mention::extract_agent_triggers(&response, &bg_channel_members);
                    let would_trigger: Vec<String> = mentioned
                        .into_iter()
                        .filter(|id| !triggered.contains(id))
                        .collect();

                    if !would_trigger.is_empty() {
                        log::info!(
                            "[send_channel_message:bg] A2A depth limit reached, skipping: {:?}",
                            would_trigger
                        );
                        let _ = bg_app.emit(
                            "agent://channel-a2a-depth-exceeded",
                            serde_json::json!({
                                "channel_id": bg_channel_id,
                                "agents": would_trigger,
                                "triggered_by": task.agent_id,
                                "depth": current_depth,
                                "max_depth": max_depth,
                            }),
                        );
                    }
                }
            }
        }

        // Emit session-complete event so the frontend knows ALL agents (including A2A chain) are done.
        let _ = bg_app.emit(
            "agent://channel-session-complete",
            serde_json::json!({
                "channel_id": bg_channel_id,
            }),
        );

        // Trigger auto-compaction check
        {
            let channels_dir = bg_workspace_root.join("channels");
            let store = ChannelStore::new(&channels_dir);
            if let Ok(ch) = store.load(&bg_channel_id) {
                if ch.messages.len() > COMPACT_THRESHOLD {
                    let _ = bg_app.emit(
                        "channel://needs-compact",
                        serde_json::json!({ "channel_id": bg_channel_id }),
                    );
                }
            }
        }
    });

    // Return the channel immediately (with just the user message)
    // plus the count of triggered agents so the frontend knows to keep listeners open.
    let agents_count = target_agents.len();
    Ok(SendChannelMessageResponse {
        channel: _channel,
        agents_triggered: agents_count,
    })
}

/// Save an agent response to a channel (called after streaming completes).
#[tauri::command]
pub fn save_channel_response(
    state: tauri::State<'_, crate::AppState>,
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
        content_blocks: None,
        timestamp: channel::now_iso(),
    };
    channel.messages.push(agent_msg);
    channel.updated_at = channel::now_iso();

    store.save(&channel).map_err(|e| format!("save failed: {e}"))?;

    Ok(channel)
}

// ---------------------------------------------------------------------------
// Channel context compaction (sliding window + auto summary)
// ---------------------------------------------------------------------------

/// Trigger auto-compaction when message count exceeds this threshold.
const COMPACT_THRESHOLD: usize = 30;
/// Number of recent messages to keep in full (not summarized).
const RECENT_KEEP_COUNT: usize = 10;

/// The prompt used to instruct the Agent to generate a conversation summary.
const SUMMARY_PROMPT: &str = r#"你是一个对话摘要助手。请将以下 Channel 对话历史压缩为一份简洁的结构化摘要。

要求：
1. 保留所有关键决策和结论
2. 保留未完成的任务和待办事项
3. 保留 Agent 之间的协作上下文（谁做了什么、约定了什么）
4. 用 [AgentName]: 标记发言人
5. 不超过 500 字

对话历史：
"#;

/// Format a slice of channel messages into a readable transcript for summarization.
fn format_messages_for_summary(
    messages: &[ChannelMessage],
    manager: &crate::workspace::manager::AgentManager,
    workspace_root: &std::path::Path,
) -> String {
    let user_name = resolve_user_name(workspace_root);
    messages
        .iter()
        .map(|msg| {
            let sender = if msg.sender_type == "user" {
                user_name.clone()
            } else {
                manager
                    .get_agent(&msg.sender_id)
                    .map(|a| a.identity.name.clone())
                    .unwrap_or_else(|| msg.sender_id.clone())
            };
            format!("[{}]: {}", sender, msg.content)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Compact (summarize) older messages in a channel.
///
/// This can be called:
/// - Automatically by `send_channel_message` when messages exceed `COMPACT_THRESHOLD`
/// - Manually via the `compact_channel` Tauri command
///
/// Uses the first available agent's runtime to generate the summary.
#[tauri::command]
pub async fn compact_channel(
    state: tauri::State<'_, crate::AppState>,
    channel_id: String,
) -> Result<Channel, String> {
    // Load channel and determine the range of messages to summarize
    let (full_prompt, summary_up_to_id, agent_id, runtime_id, workspace_path) = {
        let manager = state
            .agent_manager
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;

        let channels_dir = manager.channels_dir();
        let store = ChannelStore::new(&channels_dir);
        let channel = store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?;

        let total = channel.messages.len();
        if total <= RECENT_KEEP_COUNT {
            return Ok(channel);
        }

        let start_idx = match &channel.summary_up_to {
            Some(sid) => channel
                .messages
                .iter()
                .position(|m| m.id == *sid)
                .map(|i| i + 1)
                .unwrap_or(0),
            None => 0,
        };

        let end_idx = total.saturating_sub(RECENT_KEEP_COUNT);
        if start_idx >= end_idx {
            return Ok(channel);
        }

        let messages_to_summarize: Vec<ChannelMessage> =
            channel.messages[start_idx..end_idx].to_vec();
        if messages_to_summarize.is_empty() {
            return Ok(channel);
        }

        let summary_up_to_id = messages_to_summarize
            .last()
            .map(|m| m.id.clone())
            .unwrap_or_default();

        let agent_id = channel
            .members
            .first()
            .map(|m| m.agent_id.clone())
            .ok_or_else(|| "channel has no members".to_string())?;

        let agent = manager
            .get_agent(&agent_id)
            .ok_or_else(|| format!("agent not found: {agent_id}"))?;

        let runtime_id = agent.identity.runtime_type.runtime_id().to_string();

        let workspace = manager
            .get_workspace(&agent_id)
            .ok_or_else(|| format!("workspace not found for agent: {agent_id}"))?;
        let workspace_path = workspace.base_path().to_string_lossy().to_string();

        let transcript = format_messages_for_summary(&messages_to_summarize, &manager, manager.workspace_root());

        let full_prompt = match &channel.summary {
            Some(existing) => {
                format!(
                    "{}\n\n## 已有摘要\n\n{}\n\n## 新增对话\n\n{}",
                    SUMMARY_PROMPT, existing, transcript
                )
            }
            None => format!("{}{}", SUMMARY_PROMPT, transcript),
        };

        log::info!(
            "[compact_channel] Summarizing {} messages (idx {}..{}) for channel {}",
            messages_to_summarize.len(),
            start_idx,
            end_idx,
            channel_id
        );

        (full_prompt, summary_up_to_id, agent_id, runtime_id, workspace_path)
    };

    // Execute summary generation via Agent runtime
    let summary_text = {
        let registry = state
            .agent_runtime_registry
            .lock()
            .map_err(|e| e.to_string())?;

        let runtime = registry.get_runtime_instance(&runtime_id)?;

        if !runtime.is_ready() {
            return Err(format!(
                "Runtime {} not available for summary generation",
                runtime.name()
            ));
        }

        let params = crate::runtime::ExecuteParams {
            agent_id: agent_id.clone(),
            message: full_prompt,
            session_id: None, // Fresh session for summary
            workspace: Some(workspace_path),
            system_prompt: None,
            timeout_secs: 60,
        };

        let receiver = runtime.execute(params)?;

        // Collect the full response
        let mut result = String::new();
        while let Ok(event) = receiver.recv_timeout(std::time::Duration::from_secs(60)) {
            if event.msg_type.as_deref() == Some("assistant") && !event.text.is_empty() {
                result.push_str(&event.text);
            }
            if event.is_done {
                break;
            }
        }

        if result.is_empty() {
            return Err("Summary generation returned empty result".to_string());
        }

        result
    };

    log::info!(
        "[compact_channel] Summary generated ({} chars) for channel {}",
        summary_text.len(),
        channel_id
    );

    // Update channel with the new summary
    let updated_channel = {
        let manager = state
            .agent_manager
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;

        let channels_dir = manager.channels_dir();
        let store = ChannelStore::new(&channels_dir);
        let mut channel = store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?;

        channel.summary = Some(summary_text);
        channel.summary_up_to = Some(summary_up_to_id);
        channel.summary_updated_at = Some(channel::now_iso());
        channel.updated_at = channel::now_iso();

        store.save(&channel).map_err(|e| format!("save failed: {e}"))?;

        channel
    };

    Ok(updated_channel)
}

/// Check if a channel needs compaction and trigger it asynchronously.
///
/// Called by `send_channel_message` after adding the user message.
/// Returns immediately; compaction runs in a background thread.
pub fn maybe_auto_compact(
    app: AppHandle,
    state: tauri::State<'_, crate::AppState>,
    channel_id: String,
) {
    // Quick check: do we need compaction?
    let needs_compact = {
        let manager = match state.agent_manager.lock() {
            Ok(m) => m,
            Err(_) => return,
        };
        let channels_dir = manager.channels_dir();
        let store = ChannelStore::new(&channels_dir);
        match store.load(&channel_id) {
            Ok(ch) => ch.messages.len() > COMPACT_THRESHOLD,
            Err(_) => false,
        }
    };

    if !needs_compact {
        return;
    }

    log::info!(
        "[maybe_auto_compact] Channel {} exceeds {} messages, triggering auto-compaction",
        channel_id,
        COMPACT_THRESHOLD
    );

    // Emit event to frontend so it can call compact_channel command
    let _ = app.emit(
        "channel://needs-compact",
        serde_json::json!({
            "channel_id": channel_id,
        }),
    );
}
