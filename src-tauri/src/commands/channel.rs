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
use crate::storage::activity::{ActivityStore, ActivityType, create_entry};
use crate::storage::db_helpers;
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
/// 4. Each agent's response is saved independently
#[tauri::command]
pub async fn send_channel_message(
    app: AppHandle,
    state: tauri::State<'_, crate::AppState>,
    channel_id: String,
    message: String,
) -> Result<Channel, String> {
    // ---- Phase 1: Load channel, add user message, parse mentions ----
    let (_channel, target_agents, workspace_root) = {
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

        // Parse @mentions
        let mention_result = mention::parse_mentions(&message, &channel.members);
        let target_agents = mention::resolve_agents(&mention_result.mentions, &channel.members);

        if target_agents.is_empty() {
            return Err("channel has no agent members".to_string());
        }

        log::info!(
            "[send_channel_message] channel_id={}, parsed {} mentions, targets: {:?}",
            channel_id,
            mention_result.mentions.len(),
            target_agents
        );

        // Save channel with user message
        store.save(&channel).map_err(|e| format!("save failed: {e}"))?;

        let workspace_root = manager.workspace_root().to_path_buf();

        (channel, target_agents, workspace_root)
    };

    // ---- Phase 2: Execute each mentioned agent serially ----
    let total_agents = target_agents.len();
    for (agent_idx, agent_id) in target_agents.iter().enumerate() {
        let agent_id = agent_id.clone();
        let channel_id = channel_id.clone();
        let workspace_root = workspace_root.clone();

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

            // Build agent context via ContextBuilder
            let builder = crate::context::ContextBuilder::new(&workspace_root);
            let mut context_prefix = builder
                .build_context_prefix(&agent_id)
                .unwrap_or_default();

            // --- Sliding window + summary context ---
            // If a summary exists, include it as the "older context" prefix,
            // then only include recent messages that are NOT already summarized.
            let summary_up_to_idx = match &latest_channel.summary_up_to {
                Some(sid) => latest_channel
                    .messages
                    .iter()
                    .position(|m| m.id == *sid)
                    .map(|i| i + 1)
                    .unwrap_or(0),
                None => 0,
            };

            // Recent messages = those after the summary cutoff
            let recent_start = if summary_up_to_idx > 0 {
                summary_up_to_idx
            } else {
                // No summary: use the last N messages (legacy behavior)
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
                        "User".to_string()
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
            "[send_channel_message] executing agent {}/{}, agent_id={}, runtime={}, context: {} recent messages",
            agent_idx + 1,
            total_agents,
            agent_id,
            runtime_id,
            recent_count
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
            }),
        );

        // Start runtime execution for this agent -- route by agent's runtime_type
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
                // Emit runtime unavailable event for frontend UX
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
                message: message.clone(),
                session_id: None,
                workspace: Some(workspace_path),
                system_prompt,
                timeout_secs: 120,
            };

            runtime.execute(params)?
        };

        // Spawn thread to forward streaming events to frontend and collect response
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

        // Channel to signal completion
        let (tx_done, rx_done) = std::sync::mpsc::channel::<Result<(), String>>();

        std::thread::spawn(move || {
            let mut full_response = String::new();
            let mut result_session_id: Option<String> = None;
            let mut had_error = false;

            while let Ok(event) = receiver.recv() {
                if event.is_done {
                    result_session_id = event.session_id.clone();
                }
                if event.msg_type.as_deref() == Some("assistant") && !event.text.is_empty() {
                    full_response.push_str(&event.text);
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
                    if event.error.is_some() {
                        had_error = true;
                        log::warn!(
                            "[send_channel_message] agent {} had error: {:?}",
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
                if let Ok(mut ch) = store.load(&channel_id_clone) {
                    let agent_msg = ChannelMessage {
                        id: crate::workspace::thread::generate_id(),
                        channel_id: channel_id_clone.clone(),
                        sender_type: "agent".to_string(),
                        sender_id: agent_id_clone.clone(),
                        content: full_response.clone(),
                        timestamp: channel::now_iso(),
                    };
                    ch.messages.push(agent_msg);
                    ch.updated_at = channel::now_iso();
                    let _ = store.save(&ch);
                }

                // Emit response event for frontend
                let _ = app_clone.emit(
                    "agent://channel-response",
                    serde_json::json!({
                        "channel_id": channel_id_clone,
                        "agent_id": agent_id_clone,
                        "content": full_response,
                        "session_id": result_session_id,
                    }),
                );
            }

            // Signal completion
            let result = if had_error {
                Err("agent execution had error".to_string())
            } else {
                Ok(())
            };
            let _ = tx_done.send(result);
        });

        // Wait for this agent to complete before starting the next one
        match rx_done.recv_timeout(std::time::Duration::from_secs(300)) {
            Ok(Ok(())) => {} // Success, continue to next agent
            Ok(Err(e)) => {
                log::warn!("[send_channel_message] agent {} failed: {}, continuing", agent_id, e);
                // Continue to next agent even on failure
            }
            Err(_) => {
                log::warn!("[send_channel_message] agent {} timed out, continuing", agent_id);
                // Continue to next agent even on timeout
            }
        }
    }

    // Reload and return the final channel state
    let final_channel = {
        let manager = state
            .agent_manager
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;
        let channels_dir = manager.channels_dir();
        let store = ChannelStore::new(&channels_dir);
        store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?
    };

    // Trigger auto-compaction check (runs asynchronously, does not block response)
    maybe_auto_compact(app, state, channel_id.clone());

    Ok(final_channel)
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
) -> String {
    messages
        .iter()
        .map(|msg| {
            let sender = if msg.sender_type == "user" {
                "User".to_string()
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
    let (full_prompt, summary_up_to_id, _agent_id, runtime_id, workspace_path) = {
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

        let transcript = format_messages_for_summary(&messages_to_summarize, &manager);

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
