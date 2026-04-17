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
    _message: Message,
    auth_headers: &HashMap<String, String>,
) -> Result<mpsc::Receiver<StreamEvent>, A2AError> {
    let (tx, rx) = mpsc::channel();

    let url_owned = url.to_string();
    let auth_headers_owned = auth_headers.clone();
    let message_owned = _message.clone();

    // Spawn background thread for SSE reading
    let _handle = std::thread::spawn(move || {
        if let Err(e) = sse_reader_loop(&client, &url_owned, &message_owned, &auth_headers_owned, &tx) {
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

/// SSE reader loop: reads events from the HTTP response and forwards them.
fn sse_reader_loop(
    client: &reqwest::blocking::Client,
    url: &str,
    message: &Message,
    auth_headers: &HashMap<String, String>,
    tx: &mpsc::Sender<StreamEvent>,
) -> Result<(), A2AError> {
    let mut task = Task::new("stream-task");
    task.messages.push(message.clone());
    let request_body = serde_json::to_value(SendMessageRequest {
        task,
        stream: Some(true),
    })
    .map_err(|e| A2AError::internal_error(format!("Failed to serialize stream request: {}", e)))?;

    let mut builder = client.post(url);
    for (key, value) in auth_headers {
        builder = builder.header(key.as_str(), value.as_str());
    }

    let response = builder
        .json(&request_body)
        .send()
        .map_err(|e| A2AError::internal_error(format!("SSE connect failed: {}", e)))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        return Err(A2AError::new(
            status.as_u16() as i64,
            format!("SSE endpoint returned {}: {}", status, body),
        ));
    }

    // Read SSE events line by line
    let reader = BufReader::new(response);
    let mut current_data = String::new();
    let mut current_event_type = String::new();

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break, // Connection closed
        };

        let trimmed = line.trim();

        if trimmed.is_empty() {
            // Empty line = end of event; dispatch accumulated data
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
            // SSE spec allows multiple data lines; concatenate with newline
            if !current_data.is_empty() {
                current_data.push('\n');
            }
            current_data.push_str(data);
        } else if let Some(evt_type) = trimmed.strip_prefix("event:") {
            current_event_type = evt_type.trim().to_string();
        } else if trimmed.starts_with("id:") || trimmed.starts_with("retry:") || trimmed.starts_with(':') {
            // Ignore SSE fields we don't use (id, retry, comments)
        }
    }

    // Handle any remaining buffered data
    if !current_data.is_empty() {
        if let Some(event) = parse_sse_event(&current_event_type, &current_data) {
            let _ = tx.send(event);
        }
    }

    Ok(())
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
