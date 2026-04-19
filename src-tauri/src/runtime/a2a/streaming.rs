//! SSE (Server-Sent Events) streaming support for A2A protocol.
//!
//! Handles the `streamMessage` operation where the remote A2A endpoint
//! sends incremental messages as SSE events. Converts SSE events into
//! `StreamEvent` instances compatible with the existing runtime model.

use super::types::*;
use crate::runtime::StreamEvent;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;

/// Truncate a string to at most `max` bytes, respecting UTF-8 char boundaries.
fn char_boundary_truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Open an SSE stream to the given URL and return a receiver of StreamEvents.
///
/// The function spawns a background thread that reads SSE events from the
/// HTTP response body and forwards them as `StreamEvent` instances.
///
/// Takes an owned `reqwest::blocking::Client` so it can be moved into the
/// background thread.
pub fn open_sse_stream(
    client: reqwest::blocking::Client,
    url: &str,
    task_id: &str,
    _message: Message,
    auth_headers: &HashMap<String, String>,
) -> Result<mpsc::Receiver<StreamEvent>, A2AError> {
    let (tx, rx) = mpsc::channel();

    let url_owned = url.to_string();
    let task_id_owned = task_id.to_string();
    let auth_headers_owned = auth_headers.clone();
    let message_owned = _message.clone();

    // Spawn background thread for SSE reading
    let _handle = std::thread::spawn(move || {
        if let Err(e) = sse_reader_loop(&client, &url_owned, &task_id_owned, &message_owned, &auth_headers_owned, &tx) {
            // Send error event before closing
            let _ = tx.send(StreamEvent {
                text: String::new(),
                is_done: true,
                error: Some(format!("SSE stream error: {}", e)),
                msg_type: Some("error".to_string()),
                session_id: None,
                content_blocks: None,
            });
        }
    });

    Ok(rx)
}

/// SSE reader loop: sends sendMessage and handles the response.
///
/// Two possible response modes:
/// 1. **SSE stream**: endpoint returns `text/event-stream` with streaming events
/// 2. **Async task**: endpoint returns JSON with `task.status = WORKING`,
///    and we poll `getTask` until completion
fn sse_reader_loop(
    client: &reqwest::blocking::Client,
    url: &str,
    task_id: &str,
    message: &Message,
    auth_headers: &HashMap<String, String>,
    tx: &mpsc::Sender<StreamEvent>,
) -> Result<(), A2AError> {
    // Build JSON-RPC sendMessage request with stream=true
    let mut task = Task::new(task_id);
    task.messages.push(message.clone());
    let params = serde_json::to_value(SendMessageRequest {
        task,
        stream: Some(true),
    })
    .map_err(|e| A2AError::internal_error(format!("Failed to serialize stream request: {}", e)))?;

    let rpc_request = JsonRpcRequest::new("sendMessage", Some(params));

    let mut builder = client.post(url);
    for (key, value) in auth_headers {
        builder = builder.header(key.as_str(), value.as_str());
    }

    log::info!("[sse_reader] Sending sendMessage to {}", url);

    let response = builder
        .json(&rpc_request)
        .send()
        .map_err(|e| A2AError::internal_error(format!("SSE connect failed: {}", e)))?;

    let http_status = response.status();
    if !http_status.is_success() {
        let body = response.text().unwrap_or_default();
        log::warn!("[sse_reader] HTTP error {}: {}", http_status, char_boundary_truncate(&body, 200));
        return Err(A2AError::new(
            http_status.as_u16() as i64,
            format!("SSE endpoint returned {}: {}", http_status, body),
        ));
    }

    // Check Content-Type to determine response mode
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    if content_type.contains("text/event-stream") {
        // SSE mode: read streaming events
        log::info!("[sse_reader] SSE stream detected");
        read_sse_stream(response, tx)
    } else {
        // Non-SSE: read body and handle as async task or single response
        let body = response.text().unwrap_or_default();
        log::info!("[sse_reader] Non-SSE response ({} bytes)", body.len());
        handle_non_sse_response(client, url, task_id, &body, auth_headers, tx)
    }
}

/// Read an SSE stream from an HTTP response and forward events.
fn read_sse_stream(
    response: reqwest::blocking::Response,
    tx: &mpsc::Sender<StreamEvent>,
) -> Result<(), A2AError> {
    let reader = BufReader::new(response);
    let mut current_data = String::new();
    let mut current_event_type = String::new();

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();

        if trimmed.is_empty() {
            if !current_data.is_empty() {
                if let Some(event) = parse_sse_event(&current_event_type, &current_data) {
                    let is_done = event.is_done;
                    let _ = tx.send(event);
                    if is_done {
                        break;
                    }
                }
            }
            current_data.clear();
            current_event_type.clear();
            continue;
        }

        if let Some(data) = trimmed.strip_prefix("data:") {
            let data = data.trim();
            if !current_data.is_empty() {
                current_data.push('\n');
            }
            current_data.push_str(data);
        } else if let Some(evt_type) = trimmed.strip_prefix("event:") {
            current_event_type = evt_type.trim().to_string();
        }
    }

    if !current_data.is_empty() {
        if let Some(event) = parse_sse_event(&current_event_type, &current_data) {
            let _ = tx.send(event);
        }
    }

    Ok(())
}

/// Handle a non-SSE JSON response.
///
/// If the response contains a task with `status = WORKING`, poll `getTask`
/// until the task reaches a terminal state (COMPLETED / FAILED / CANCELED).
/// Otherwise, treat the response as a single-shot result.
fn handle_non_sse_response(
    client: &reqwest::blocking::Client,
    url: &str,
    task_id: &str,
    body: &str,
    auth_headers: &HashMap<String, String>,
    tx: &mpsc::Sender<StreamEvent>,
) -> Result<(), A2AError> {
    // Try to parse as JSON-RPC response containing a Task
    let json: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| A2AError::internal_error(format!("Failed to parse response: {}", e)))?;

    // Check for JSON-RPC error
    if let Some(error) = json.get("error") {
        let msg = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
        let _ = tx.send(StreamEvent {
            text: String::new(),
            is_done: true,
            error: Some(msg.to_string()),
            msg_type: Some("error".to_string()),
            session_id: None,
            content_blocks: None,
        });
        return Ok(());
    }

    // Extract task from result
    let task_json = json.get("result")
        .and_then(|r| r.get("task"))
        .cloned();

    let task_status = task_json.as_ref()
        .and_then(|t| t.get("status"))
        .and_then(|s| s.as_str())
        .unwrap_or("");

    // If task is WORKING/SUBMITTED, switch to polling mode
    if task_status == "WORKING" || task_status == "SUBMITTED" {
        log::info!("[sse_reader] Task {} status={}, switching to poll mode", task_id, task_status);
        return poll_task_until_done(client, url, task_id, auth_headers, tx);
    }

    // Task is already in a terminal state — extract messages and forward
    if let Some(task) = task_json {
        forward_task_result(&task, tx);
    }

    Ok(())
}

/// Poll `getTask` via JSON-RPC until the task reaches a terminal state.
fn poll_task_until_done(
    client: &reqwest::blocking::Client,
    url: &str,
    task_id: &str,
    auth_headers: &HashMap<String, String>,
    tx: &mpsc::Sender<StreamEvent>,
) -> Result<(), A2AError> {
    // Use a short-timeout client for polling (don't reuse the 30s one)
    let poll_client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| A2AError::internal_error(format!("Failed to build poll client: {}", e)))?;

    let poll_interval = std::time::Duration::from_secs(3);
    let max_poll_duration = std::time::Duration::from_secs(300); // 5 min timeout
    let start = std::time::Instant::now();
    let mut poll_count = 0u32;

    loop {
        std::thread::sleep(poll_interval);

        if start.elapsed() > max_poll_duration {
            let _ = tx.send(StreamEvent {
                text: String::new(),
                is_done: true,
                error: Some("Polling timeout: remote task did not complete".to_string()),
                msg_type: Some("error".to_string()),
                session_id: None,
                content_blocks: None,
            });
            return Ok(());
        }

        poll_count += 1;

        // Build getTask JSON-RPC request
        let rpc = JsonRpcRequest::new("getTask", Some(serde_json::json!({
            "id": task_id
        })));

        let mut builder = poll_client.post(url);
        for (key, value) in auth_headers {
            builder = builder.header(key.as_str(), value.as_str());
        }

        log::info!("[sse_reader] Poll #{} for task {}", poll_count, task_id);

        let response = builder
            .json(&rpc)
            .send()
            .map_err(|e| A2AError::internal_error(format!("getTask poll failed: {}", e)))?;

        if !response.status().is_success() {
            log::warn!("[sse_reader] getTask poll HTTP error: {}", response.status());
            continue;
        }

        let body = response.text().unwrap_or_default();
        log::info!(
            "[sse_reader] Poll #{} response ({} bytes): {}",
            poll_count,
            body.len(),
            char_boundary_truncate(&body, 600)
        );

        let json: serde_json::Value = match serde_json::from_str(&body) {
            Ok(j) => j,
            Err(e) => {
                log::warn!("[sse_reader] Poll #{} JSON parse error: {}", poll_count, e);
                continue;
            }
        };

        // Check for JSON-RPC error
        if let Some(err) = json.get("error") {
            let msg = err.get("message").and_then(|m| m.as_str()).unwrap_or("unknown");
            log::warn!("[sse_reader] Poll #{} RPC error: {}", poll_count, msg);
            continue;
        }

        let task_json = match json.get("result").and_then(|r| r.get("task")) {
            Some(t) => t,
            None => {
                log::warn!("[sse_reader] Poll #{} no result.task in response", poll_count);
                continue;
            }
        };

        let status_str = task_json.get("status").and_then(|s| s.as_str()).unwrap_or("UNKNOWN");

        log::info!("[sse_reader] Poll task {} status={}", task_id, status_str);

        // Check if task has agent messages even while WORKING — treat as done
        if status_str == "WORKING" || status_str == "SUBMITTED" {
            if has_agent_messages(task_json) {
                log::info!("[sse_reader] Task {} has agent messages, treating as complete", task_id);
                forward_task_result(task_json, tx);
                return Ok(());
            }
            // Send progress event to frontend so user knows we're still waiting
            if poll_count % 3 == 0 {
                let elapsed = start.elapsed().as_secs();
                let _ = tx.send(StreamEvent {
                    text: format!("等待远程 Agent 响应... ({}s)", elapsed),
                    is_done: false,
                    error: None,
                    msg_type: Some("system".to_string()),
                    session_id: None,
                    content_blocks: None,
                });
            }
            // Still working, continue polling
            continue;
        }

        // Check if terminal
        match status_str {
            "COMPLETED" | "CANCELED" => {
                forward_task_result(task_json, tx);
                return Ok(());
            }
            "FAILED" => {
                let error_msg = task_json.get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Remote task failed");
                let _ = tx.send(StreamEvent {
                    text: String::new(),
                    is_done: true,
                    error: Some(error_msg.to_string()),
                    msg_type: Some("error".to_string()),
                    session_id: None,
                    content_blocks: None,
                });
                return Ok(());
            }
            _ => {
                // Still working, continue polling
            }
        }
    }
}

/// Extract messages from a completed Task and forward as StreamEvents.
///
/// Supports multiple message formats:
/// - A2A standard: `{"role":"agent","parts":[{"type":"text","text":"..."}]}`
/// - Simple format: `{"role":"agent","content":"..."}`
/// - OpenAI-style: `{"role":"assistant","content":"..."}`
fn forward_task_result(
    task_json: &serde_json::Value,
    tx: &mpsc::Sender<StreamEvent>,
) {
    let session_id = task_json.get("session_id")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    // Extract agent messages from the task
    if let Some(messages) = task_json.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
            let is_agent = role == "agent" || role == "assistant";
            if !is_agent {
                continue;
            }

            let text = extract_message_text(msg);
            if !text.is_empty() {
                let _ = tx.send(StreamEvent {
                    text,
                    is_done: false,
                    error: None,
                    msg_type: Some("assistant".to_string()),
                    session_id: session_id.clone(),
                    content_blocks: None,
                });
            }
        }
    }

    // Send done event
    let _ = tx.send(StreamEvent {
        text: String::new(),
        is_done: true,
        error: None,
        msg_type: None,
        session_id,
        content_blocks: None,
    });
}

/// Check if the task JSON contains agent messages with non-empty text.
///
/// Supports multiple message formats:
/// - A2A standard: `{"role":"agent","parts":[{"type":"text","text":"..."}]}`
/// - Simple format: `{"role":"agent","content":"..."}`
/// - OpenAI-style: `{"role":"assistant","content":"..."}`
fn has_agent_messages(task_json: &serde_json::Value) -> bool {
    let messages = match task_json.get("messages").and_then(|m| m.as_array()) {
        Some(msgs) => msgs,
        None => return false,
    };
    messages.iter().any(|msg| {
        let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let is_agent = role == "agent" || role == "assistant";
        if !is_agent {
            return false;
        }
        // Check A2A parts format
        if let Some(parts) = msg.get("parts").and_then(|p| p.as_array()) {
            return parts.iter().any(|p| {
                p.get("type").and_then(|t| t.as_str()) == Some("text")
                    && p.get("text").and_then(|t| t.as_str()).map_or(false, |t| !t.is_empty())
            });
        }
        // Check simple content string format
        if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
            return !content.is_empty();
        }
        // Check content array format (OpenAI style)
        if let Some(parts) = msg.get("content").and_then(|c| c.as_array()) {
            return parts.iter().any(|p| {
                p.get("type").and_then(|t| t.as_str()) == Some("text")
                    && p.get("text").and_then(|t| t.as_str()).map_or(false, |t| !t.is_empty())
            });
        }
        false
    })
}

/// Parse a single SSE event's data payload into a StreamEvent.
///
/// The data field may contain:
/// - An A2A Message JSON object
/// - A Task status update
/// - A plain text string
fn parse_sse_event(event_type: &str, data: &str) -> Option<StreamEvent> {
    // Try parsing as A2A Message
    if let Ok(msg) = serde_json::from_str::<Message>(data) {
        let text = extract_text_from_parts(&msg.parts);
        let is_done = event_type == "done" || msg.role == MessageRole::System;
        return Some(StreamEvent {
            text,
            is_done,
            error: None,
            msg_type: Some(match msg.role {
                MessageRole::User => "user",
                MessageRole::Agent => "assistant",
                MessageRole::System => "system",
            }.to_string()),
            session_id: None,
            content_blocks: None,
        });
    }

    // Try parsing as Task status update
    if let Ok(task) = serde_json::from_str::<Task>(data) {
        let is_done = task.status.is_terminal();
        return Some(StreamEvent {
            text: format!("Task status: {}", task.status.as_str()),
            is_done,
            error: if task.status == TaskStatus::Failed {
                Some("Task failed".to_string())
            } else {
                None
            },
            msg_type: Some("system".to_string()),
            session_id: task.session_id,
            content_blocks: None,
        });
    }

    // Fallback: treat data as plain text
    if !data.is_empty() {
        let is_done = event_type == "done";
        return Some(StreamEvent {
            text: data.to_string(),
            is_done,
            error: None,
            msg_type: Some("raw".to_string()),
            session_id: None,
            content_blocks: None,
        });
    }

    None
}

/// Extract text from a message JSON value, supporting multiple formats.
fn extract_message_text(msg: &serde_json::Value) -> String {
    // 1. A2A standard: parts array with type/text objects
    if let Some(parts) = msg.get("parts").and_then(|p| p.as_array()) {
        let text: String = parts.iter()
            .filter_map(|p| {
                if p.get("type").and_then(|t| t.as_str()) == Some("text") {
                    p.get("text").and_then(|t| t.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");
        if !text.is_empty() {
            return text;
        }
    }
    // 2. Simple content string
    if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
        if !content.is_empty() {
            return content.to_string();
        }
    }
    // 3. Content array (OpenAI style)
    if let Some(parts) = msg.get("content").and_then(|c| c.as_array()) {
        let text: String = parts.iter()
            .filter_map(|p| {
                if p.get("type").and_then(|t| t.as_str()) == Some("text") {
                    p.get("text").and_then(|t| t.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");
        if !text.is_empty() {
            return text;
        }
    }
    String::new()
}

/// Extract concatenated text from a list of Parts.
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

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_from_parts() {
        let parts = vec![
            Part::Text {
                text: "Hello ".into(),
            },
            Part::Data {
                data: serde_json::json!({"key": "val"}),
            },
            Part::Text {
                text: "World".into(),
            },
        ];
        assert_eq!(extract_text_from_parts(&parts), "Hello World");
    }

    #[test]
    fn test_parse_sse_event_agent_message() {
        let data = r#"{"role":"agent","parts":[{"type":"text","text":"Hello!"}]}"#;
        let event = parse_sse_event("message", data).unwrap();
        assert_eq!(event.text, "Hello!");
        assert_eq!(event.msg_type.as_deref(), Some("assistant"));
        assert!(!event.is_done);
    }

    #[test]
    fn test_parse_sse_event_done_event() {
        let data = r#"{"role":"system","parts":[{"type":"text","text":"Done"}]}"#;
        let event = parse_sse_event("done", data).unwrap();
        assert!(event.is_done);
    }

    #[test]
    fn test_parse_sse_event_task_update() {
        let data = r#"{"id":"t-1","status":"COMPLETED","messages":[],"artifacts":[]}"#;
        let event = parse_sse_event("status", data).unwrap();
        assert!(event.is_done);
        assert!(event.text.contains("COMPLETED"));
    }

    #[test]
    fn test_parse_sse_event_plain_text() {
        let event = parse_sse_event("message", "plain text data").unwrap();
        assert_eq!(event.text, "plain text data");
        assert_eq!(event.msg_type.as_deref(), Some("raw"));
        assert!(!event.is_done);
    }

    #[test]
    fn test_parse_sse_event_done_plain_text() {
        let event = parse_sse_event("done", "final chunk").unwrap();
        assert!(event.is_done);
    }

    #[test]
    fn test_parse_sse_event_empty() {
        assert!(parse_sse_event("", "").is_none());
    }

    #[test]
    fn test_parse_sse_event_failed_task() {
        let data = r#"{"id":"t-1","status":"FAILED","messages":[],"artifacts":[]}"#;
        let event = parse_sse_event("status", data).unwrap();
        assert!(event.is_done);
        assert!(event.error.is_some());
    }
}
