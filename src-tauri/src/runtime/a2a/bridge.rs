//! Bridge between existing StreamEvent and A2A Message types.
//!
//! Provides bidirectional conversion between the internal `StreamEvent`
//! model (used by Claude Code / Codex runtimes) and the A2A `Message` /
//! `Artifact` types defined in the A2A protocol.
//!
//! This enables remote A2A endpoints to produce events that look identical
//! to local CLI events from the frontend's perspective.

use super::types::*;
use crate::runtime::StreamEvent;

/// Convert a `StreamEvent` (from a local CLI runtime) into an A2A `Message`.
///
/// Maps the internal event types to A2A message roles:
/// - "assistant" → Agent role
/// - "user"      → User role
/// - "system"    → System role
/// - "result"    → Agent role (with text output)
/// - "raw"       → Agent role (raw stdout)
/// - "stderr"    → Agent role (error output)
/// - "timeout"   → System role
///
/// Content blocks containing tool_use / tool_result are extracted into
/// `Artifact` instances attached to the message metadata.
pub fn stream_event_to_a2a_message(event: &StreamEvent) -> Message {
    let role = match event.msg_type.as_deref() {
        Some("user") => MessageRole::User,
        Some("system") | Some("timeout") => MessageRole::System,
        // "assistant", "result", "raw", "stderr", and others default to Agent
        _ => MessageRole::Agent,
    };

    let mut parts = Vec::new();

    // Primary text content
    if !event.text.is_empty() {
        parts.push(Part::Text {
            text: event.text.clone(),
        });
    }

    // Error as a separate text part
    if let Some(ref error) = event.error {
        parts.push(Part::Text {
            text: format!("[Error] {}", error),
        });
    }

    // Extract content_blocks into artifacts stored in metadata
    let mut metadata = serde_json::Map::new();

    if let Some(ref content_blocks) = event.content_blocks {
        let artifacts = extract_artifacts_from_content_blocks(content_blocks);
        if !artifacts.is_empty() {
            metadata.insert(
                "artifacts".to_string(),
                serde_json::to_value(&artifacts).unwrap_or_default(),
            );
        }
    }

    if let Some(ref session_id) = event.session_id {
        metadata.insert(
            "session_id".to_string(),
            serde_json::Value::String(session_id.clone()),
        );
    }

    if event.is_done {
        metadata.insert(
            "is_done".to_string(),
            serde_json::Value::Bool(true),
        );
    }

    // Map msg_type into metadata for roundtrip fidelity
    if let Some(ref msg_type) = event.msg_type {
        metadata.insert(
            "original_msg_type".to_string(),
            serde_json::Value::String(msg_type.clone()),
        );
    }

    // Ensure at least one part exists
    if parts.is_empty() {
        parts.push(Part::Text {
            text: String::new(),
        });
    }

    Message {
        role,
        parts,
        context: None,
        metadata: if metadata.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(metadata))
        },
    }
}

/// Convert an A2A `Message` back into a `StreamEvent`.
///
/// This is the reverse of `stream_event_to_a2a_message`, used when
/// receiving messages from a remote A2A endpoint and feeding them
/// into the existing UI rendering pipeline.
pub fn a2a_message_to_stream_event(message: &Message) -> StreamEvent {
    let text = extract_text_from_parts(&message.parts);
    let error = message.parts.iter().find_map(|p| match p {
        Part::Text { text } if text.starts_with("[Error] ") => {
            Some(text.trim_start_matches("[Error] ").to_string())
        }
        _ => None,
    });

    let msg_type = message.metadata.as_ref().and_then(|m| {
        m.get("original_msg_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| match message.role {
                MessageRole::User => "user".to_string(),
                MessageRole::Agent => "assistant".to_string(),
                MessageRole::System => "system".to_string(),
            })
            .into()
    }).or_else(|| {
        match message.role {
            MessageRole::User => Some("user".to_string()),
            MessageRole::Agent => Some("assistant".to_string()),
            MessageRole::System => Some("system".to_string()),
        }
    });

    let session_id = message.metadata.as_ref().and_then(|m| {
        m.get("session_id").and_then(|v| v.as_str()).map(String::from)
    });

    let is_done = message.metadata.as_ref().and_then(|m| {
        m.get("is_done").and_then(|v| v.as_bool())
    }).unwrap_or(false);

    let content_blocks = message.metadata.as_ref().and_then(|m| {
        m.get("artifacts").cloned()
    });

    StreamEvent {
        text,
        is_done,
        error,
        msg_type,
        session_id,
        content_blocks,
    }
}

/// Map A2A `TaskStatus` to a simplified execution state string.
///
/// Used for mapping between the A2A task lifecycle and the internal
/// agent execution status model.
pub fn task_status_to_exec_status(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Submitted => "pending",
        TaskStatus::Working => "running",
        TaskStatus::Completed => "completed",
        TaskStatus::Failed => "failed",
        TaskStatus::Canceled => "canceled",
        TaskStatus::Rejected => "rejected",
        TaskStatus::InputRequired => "input_required",
        TaskStatus::AuthRequired => "auth_required",
    }
}

/// Map an internal execution state string to an A2A `TaskStatus`.
pub fn exec_status_to_task_status(status: &str) -> TaskStatus {
    match status {
        "pending" | "queued" => TaskStatus::Submitted,
        "running" | "executing" => TaskStatus::Working,
        "completed" | "success" | "done" => TaskStatus::Completed,
        "failed" | "error" => TaskStatus::Failed,
        "canceled" | "cancelled" => TaskStatus::Canceled,
        "rejected" => TaskStatus::Rejected,
        "input_required" | "waiting_input" => TaskStatus::InputRequired,
        "auth_required" => TaskStatus::AuthRequired,
        _ => TaskStatus::Submitted,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Extract text content from message parts.
fn extract_text_from_parts(parts: &[Part]) -> String {
    parts
        .iter()
        .filter_map(|p| match p {
            Part::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Extract Artifact instances from content_blocks JSON.
///
/// Content blocks from Claude Code CLI follow the structure:
/// ```json
/// [{"type": "tool_use", "name": "...", "input": {...}}, ...]
/// ```
fn extract_artifacts_from_content_blocks(
    content_blocks: &serde_json::Value,
) -> Vec<Artifact> {
    let blocks = match content_blocks.as_array() {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    let mut artifacts = Vec::new();
    let mut idx = 0u64;

    for block in blocks {
        let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match block_type {
            "tool_use" => {
                let name = block
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown_tool");
                let input = block.get("input").cloned().unwrap_or_default();

                artifacts.push(Artifact {
                    id: format!("tool-use-{}", idx),
                    name: Some(format!("Tool: {}", name)),
                    description: Some(format!("Tool call: {}", name)),
                    parts: vec![Part::Data { data: input }],
                    created_at: chrono::Utc::now().to_rfc3339(),
                });
                idx += 1;
            }
            "tool_result" => {
                let content = block.get("content").cloned().unwrap_or_default();

                artifacts.push(Artifact {
                    id: format!("tool-result-{}", idx),
                    name: Some("Tool Result".into()),
                    description: None,
                    parts: vec![Part::Data { data: content }],
                    created_at: chrono::Utc::now().to_rfc3339(),
                });
                idx += 1;
            }
            _ => {}
        }
    }

    artifacts
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_event_to_a2a_message_assistant() {
        let event = StreamEvent {
            text: "Hello from Claude".into(),
            is_done: false,
            error: None,
            msg_type: Some("assistant".into()),
            session_id: Some("sess-1".into()),
            content_blocks: None,
        };
        let msg = stream_event_to_a2a_message(&event);
        assert_eq!(msg.role, MessageRole::Agent);
        assert_eq!(msg.parts.len(), 1);
        assert_eq!(msg.metadata.as_ref().unwrap()["session_id"], "sess-1");
    }

    #[test]
    fn test_stream_event_to_a2a_message_user() {
        let event = StreamEvent {
            text: "Do something".into(),
            is_done: false,
            error: None,
            msg_type: Some("user".into()),
            session_id: None,
            content_blocks: None,
        };
        let msg = stream_event_to_a2a_message(&event);
        assert_eq!(msg.role, MessageRole::User);
    }

    #[test]
    fn test_stream_event_to_a2a_message_system() {
        let event = StreamEvent {
            text: "System notice".into(),
            is_done: false,
            error: None,
            msg_type: Some("system".into()),
            session_id: None,
            content_blocks: None,
        };
        let msg = stream_event_to_a2a_message(&event);
        assert_eq!(msg.role, MessageRole::System);
    }

    #[test]
    fn test_stream_event_to_a2a_message_with_error() {
        let event = StreamEvent {
            text: "Partial output".into(),
            is_done: true,
            error: Some("Process crashed".into()),
            msg_type: Some("result".into()),
            session_id: None,
            content_blocks: None,
        };
        let msg = stream_event_to_a2a_message(&event);
        assert_eq!(msg.role, MessageRole::Agent);
        assert_eq!(msg.parts.len(), 2);
        let error_text = &msg.parts[1];
        match error_text {
            Part::Text { text } => assert!(text.contains("Process crashed")),
            _ => panic!("Expected Text part"),
        }
    }

    #[test]
    fn test_stream_event_to_a2a_message_with_content_blocks() {
        let event = StreamEvent {
            text: "Working".into(),
            is_done: false,
            error: None,
            msg_type: Some("assistant".into()),
            session_id: None,
            content_blocks: Some(serde_json::json!([
                {"type": "tool_use", "name": "read_file", "input": {"path": "/tmp/a.txt"}},
                {"type": "tool_result", "content": "file contents here"}
            ])),
        };
        let msg = stream_event_to_a2a_message(&event);
        let meta = msg.metadata.as_ref().unwrap();
        assert!(meta.get("artifacts").is_some());
        let artifacts = meta.get("artifacts").unwrap().as_array().unwrap();
        assert_eq!(artifacts.len(), 2);
    }

    #[test]
    fn test_stream_event_to_a2a_message_timeout() {
        let event = StreamEvent {
            text: "".into(),
            is_done: true,
            error: None,
            msg_type: Some("timeout".into()),
            session_id: None,
            content_blocks: None,
        };
        let msg = stream_event_to_a2a_message(&event);
        assert_eq!(msg.role, MessageRole::System);
        assert!(msg.metadata.as_ref().unwrap()["is_done"].as_bool().unwrap());
    }

    #[test]
    fn test_a2a_message_to_stream_event_agent() {
        let msg = Message {
            role: MessageRole::Agent,
            parts: vec![Part::Text {
                text: "Hello!".into(),
            }],
            context: None,
            metadata: Some(serde_json::json!({
                "original_msg_type": "assistant",
                "session_id": "sess-1"
            })),
        };
        let event = a2a_message_to_stream_event(&msg);
        assert_eq!(event.text, "Hello!");
        assert_eq!(event.msg_type.as_deref(), Some("assistant"));
        assert_eq!(event.session_id.as_deref(), Some("sess-1"));
    }

    #[test]
    fn test_a2a_message_to_stream_event_user() {
        let msg = Message {
            role: MessageRole::User,
            parts: vec![Part::Text {
                text: "Do this".into(),
            }],
            context: None,
            metadata: None,
        };
        let event = a2a_message_to_stream_event(&msg);
        assert_eq!(event.text, "Do this");
        assert_eq!(event.msg_type.as_deref(), Some("user"));
    }

    #[test]
    fn test_a2a_message_to_stream_event_with_error() {
        let msg = Message {
            role: MessageRole::Agent,
            parts: vec![
                Part::Text {
                    text: "Output".into(),
                },
                Part::Text {
                    text: "[Error] Something went wrong".into(),
                },
            ],
            context: None,
            metadata: None,
        };
        let event = a2a_message_to_stream_event(&msg);
        assert_eq!(event.error.as_deref(), Some("Something went wrong"));
    }

    #[test]
    fn test_roundtrip_assistant_event() {
        let original = StreamEvent {
            text: "Test output".into(),
            is_done: false,
            error: None,
            msg_type: Some("assistant".into()),
            session_id: Some("sess-rt".into()),
            content_blocks: None,
        };
        let msg = stream_event_to_a2a_message(&original);
        let back = a2a_message_to_stream_event(&msg);
        assert_eq!(original.text, back.text);
        assert_eq!(original.msg_type, back.msg_type);
        assert_eq!(original.session_id, back.session_id);
        assert_eq!(original.is_done, back.is_done);
    }

    #[test]
    fn test_roundtrip_system_done_event() {
        let original = StreamEvent {
            text: "Complete".into(),
            is_done: true,
            error: None,
            msg_type: Some("system".into()),
            session_id: None,
            content_blocks: None,
        };
        let msg = stream_event_to_a2a_message(&original);
        let back = a2a_message_to_stream_event(&msg);
        assert_eq!(original.text, back.text);
        assert!(back.is_done);
    }

    #[test]
    fn test_task_status_mapping_roundtrip() {
        let mappings = vec![
            (TaskStatus::Submitted, "pending"),
            (TaskStatus::Working, "running"),
            (TaskStatus::Completed, "completed"),
            (TaskStatus::Failed, "failed"),
            (TaskStatus::Canceled, "canceled"),
            (TaskStatus::Rejected, "rejected"),
            (TaskStatus::InputRequired, "input_required"),
            (TaskStatus::AuthRequired, "auth_required"),
        ];
        for (status, exec) in mappings {
            assert_eq!(task_status_to_exec_status(&status), exec);
            assert_eq!(exec_status_to_task_status(exec), status);
        }
    }

    #[test]
    fn test_exec_status_aliases() {
        assert_eq!(exec_status_to_task_status("queued"), TaskStatus::Submitted);
        assert_eq!(exec_status_to_task_status("success"), TaskStatus::Completed);
        assert_eq!(exec_status_to_task_status("done"), TaskStatus::Completed);
        assert_eq!(exec_status_to_task_status("error"), TaskStatus::Failed);
        assert_eq!(exec_status_to_task_status("cancelled"), TaskStatus::Canceled);
    }
}
