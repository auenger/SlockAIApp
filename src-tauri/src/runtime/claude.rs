//! Claude Code CLI runtime implementation.
//!
//! Wraps the Claude Code CLI (`claude`) as an AgentRuntime.
//! Uses `--output-format stream-json` for streaming responses
//! and `--resume` for session continuity.

use super::{
    AgentCapability, AgentRuntime, AgentRuntimeInfo, AgentRuntimeStatus, ExecuteParams,
    RuntimeDetector, RuntimeType, StreamEvent,
};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ===========================================================================
// ClaudeCodeRuntime
// ===========================================================================

/// Claude Code CLI runtime implementation.
#[derive(Default)]
pub struct ClaudeCodeRuntime;

impl ClaudeCodeRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl AgentRuntime for ClaudeCodeRuntime {
    fn id(&self) -> &str {
        "claude-code"
    }

    fn name(&self) -> &str {
        "Claude Code"
    }

    fn runtime_category(&self) -> &str {
        "cli"
    }

    fn typed_runtime_type(&self) -> RuntimeType {
        RuntimeType::ClaudeCode
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Streaming,
            AgentCapability::Sessions,
            AgentCapability::ToolUse,
            AgentCapability::StructuredOutput,
        ]
    }

    fn install_hint(&self) -> String {
        "npm install -g @anthropic-ai/claude-code".to_string()
    }

    fn binary_name(&self) -> &str {
        "claude"
    }

    fn detect(&self) -> Result<Option<(String, String)>, String> {
        let path = RuntimeDetector::find_command("claude");
        match path {
            Some(p) => {
                let version =
                    RuntimeDetector::get_version("claude").unwrap_or_else(|| "unknown".to_string());
                Ok(Some((p, version)))
            }
            None => Ok(None),
        }
    }

    fn health_check(&self) -> AgentRuntimeStatus {
        match self.detect() {
            Ok(Some(_)) => AgentRuntimeStatus::Available,
            _ => AgentRuntimeStatus::Unhealthy,
        }
    }

    fn info(&self) -> AgentRuntimeInfo {
        match self.detect() {
            Ok(Some((path, version))) => AgentRuntimeInfo {
                id: self.id().to_string(),
                name: self.name().to_string(),
                runtime_category: self.runtime_category().to_string(),
                runtime_type: self.typed_runtime_type(),
                status: AgentRuntimeStatus::Available.as_str().to_string(),
                version: Some(version),
                install_path: Some(path),
                capabilities: self.capabilities(),
                install_hint: self.install_hint(),
                binary_name: Some(self.binary_name().to_string()),
            },
            _ => AgentRuntimeInfo {
                id: self.id().to_string(),
                name: self.name().to_string(),
                runtime_category: self.runtime_category().to_string(),
                runtime_type: self.typed_runtime_type(),
                status: AgentRuntimeStatus::NotInstalled.as_str().to_string(),
                version: None,
                install_path: None,
                capabilities: self.capabilities(),
                install_hint: self.install_hint(),
                binary_name: Some(self.binary_name().to_string()),
            },
        }
    }

    fn is_ready(&self) -> bool {
        matches!(self.health_check(), AgentRuntimeStatus::Available)
    }

    fn execute(
        &self,
        params: ExecuteParams,
    ) -> Result<std::sync::mpsc::Receiver<StreamEvent>, String> {
        use std::time::Duration;

        // Pre-flight: verify CLI is available
        if !self.is_ready() {
            return Err(
                "Claude Code CLI not found or not healthy. Please install: npm install -g @anthropic-ai/claude-code"
                    .to_string(),
            );
        }

        // Build CLI arguments
        let mut args: Vec<String> = vec![
            "--print".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
            "--dangerously-skip-permissions".to_string(),
        ];

        if let Some(ref sid) = params.session_id {
            args.push("--resume".to_string());
            args.push(sid.clone());
        }

        if let Some(ref sp) = params.system_prompt {
            args.push("--append-system-prompt".to_string());
            args.push(sp.clone());
        }

        if let Some(ref ws) = params.workspace {
            if !ws.is_empty() {
                args.push("--add-dir".to_string());
                args.push(ws.clone());
            }
        }

        args.push("--".to_string());
        args.push(params.message.clone());

        log::info!(
            "[ClaudeCodeRuntime] Spawning CLI with args: {:?}",
            args.iter().take(8).collect::<Vec<_>>()
        );
        log::info!(
            "[ClaudeCodeRuntime] session_id={:?}, workspace={:?}",
            params.session_id, params.workspace
        );

        // Spawn CLI process
        let mut cmd = Command::new("claude");
        if let Some(ref ws) = params.workspace {
            if !ws.is_empty() {
                cmd.current_dir(ws);
            }
        }
        let mut child = cmd
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn Claude CLI: {}", e))?;

        let stdout_handle = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to get stdout handle".to_string())?;
        let stderr_handle = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to get stderr handle".to_string())?;

        let (tx, rx) = std::sync::mpsc::channel();

        // Shared state for idle watchdog
        let last_active = Arc::new(AtomicU64::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ));
        let process_done = Arc::new(AtomicBool::new(false));

        // --- stdout reader thread ---
        let tx_stdout = tx.clone();
        let last_active_stdout = last_active.clone();
        let process_done_stdout = process_done.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout_handle);
            let mut got_result = false;

            for line in reader.lines() {
                match line {
                    Ok(text) => {
                        // Update activity timestamp for idle watchdog
                        last_active_stdout.store(
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            Ordering::Relaxed,
                        );

                        let trimmed = text.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        // Try to parse as JSON (stream-json format)
                        match serde_json::from_str::<serde_json::Value>(trimmed) {
                            Ok(json_obj) => {
                                let msg_type = json_obj
                                    .get("type")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();

                                let response_session_id = json_obj
                                    .get("session_id")
                                    .and_then(|s| s.as_str())
                                    .map(|s| s.to_string());

                                let is_result = msg_type == "result";
                                if is_result {
                                    got_result = true;
                                }

                                // Extract text content based on message type
                                fn extract_text_content(val: &serde_json::Value) -> String {
                                    if let Some(s) = val.as_str() {
                                        s.to_string()
                                    } else if let Some(arr) = val.as_array() {
                                        arr.iter()
                                            .filter_map(|item| {
                                                if item.get("type").and_then(|t| t.as_str())
                                                    == Some("text")
                                                {
                                                    item.get("text")
                                                        .and_then(|t| t.as_str())
                                                        .map(|s| s.to_string())
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                            .join("")
                                    } else {
                                        String::new()
                                    }
                                }

                                // Extract structured content blocks (including tool_use, tool_result)
                                fn extract_structured_blocks(val: &serde_json::Value) -> Option<serde_json::Value> {
                                    if val.is_array() {
                                        let blocks: Vec<serde_json::Value> = val.as_array().unwrap().iter()
                                            .filter(|item| {
                                                let btype = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                                btype == "tool_use" || btype == "tool_result"
                                            })
                                            .cloned()
                                            .collect();
                                        if blocks.is_empty() {
                                            None
                                        } else {
                                            Some(serde_json::Value::Array(blocks))
                                        }
                                    } else {
                                        None
                                    }
                                }

                                let content_val = if msg_type == "assistant" || msg_type == "user" {
                                    // --verbose: content is nested under "message" key
                                    if let Some(msg_obj) = json_obj.get("message") {
                                        msg_obj.get("content")
                                    } else {
                                        json_obj.get("content")
                                    }
                                } else {
                                    None
                                };

                                let text = content_val
                                    .map(extract_text_content)
                                    .unwrap_or_else(|| {
                                        if is_result {
                                            String::new()
                                        } else if msg_type == "system" {
                                            // Produce a meaningful status string for system events
                                            let subtype = json_obj
                                                .get("subtype")
                                                .and_then(|s| s.as_str())
                                                .unwrap_or("");
                                            if subtype == "init" {
                                                let model = json_obj
                                                    .get("model")
                                                    .and_then(|m| m.as_str())
                                                    .unwrap_or("");
                                                if model.is_empty() {
                                                    "Session initialized".to_string()
                                                } else {
                                                    format!("Session initialized · {}", model)
                                                }
                                            } else {
                                                format!("System: {}", subtype)
                                            }
                                        } else {
                                            trimmed.to_string()
                                        }
                                    });

                                // Extract structured content blocks (tool_use, tool_result)
                                let content_blocks = content_val.and_then(extract_structured_blocks);

                                // Check for error in result messages
                                let error = if is_result {
                                    let subtype = json_obj
                                        .get("subtype")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or("");
                                    if subtype == "error" {
                                        json_obj
                                            .get("error")
                                            .and_then(|e| e.as_str())
                                            .map(|s| s.to_string())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                let event = StreamEvent {
                                    text,
                                    is_done: is_result,
                                    error,
                                    msg_type: Some(msg_type),
                                    session_id: response_session_id,
                                    content_blocks,
                                };

                                if tx_stdout.send(event).is_err() {
                                    break;
                                }
                            }
                            Err(_) => {
                                // Non-JSON line, emit as raw text
                                let event = StreamEvent {
                                    text: trimmed.to_string(),
                                    is_done: false,
                                    error: None,
                                    content_blocks: None,
                                    msg_type: Some("raw".to_string()),
                                    session_id: None,
                                };
                                if tx_stdout.send(event).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => break, // EOF
                }
            }

            // Mark process as done for idle watchdog
            process_done_stdout.store(true, Ordering::Relaxed);

            // Only send process_exit is_done if we never received a result
            if !got_result {
                let _ = tx_stdout.send(StreamEvent {
                    text: String::new(),
                    is_done: true,
                    error: None,
                    content_blocks: None,
                    msg_type: Some("process_exit".to_string()),
                    session_id: None,
                });
            }
        });

        // --- stderr reader thread ---
        let tx_stderr = tx.clone();
        let last_active_stderr = last_active.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr_handle);
            for line in reader.lines() {
                match line {
                    Ok(text) => {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            // Update activity timestamp
                            last_active_stderr.store(
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs(),
                                Ordering::Relaxed,
                            );
                            log::warn!("[ClaudeCodeRuntime] stderr: {}", trimmed);
                            let event = StreamEvent {
                                text: String::new(),
                                is_done: false,
                                error: Some(format!("CLI stderr: {}", trimmed)),
                                content_blocks: None,
                                msg_type: Some("stderr".to_string()),
                                session_id: None,
                            };
                            if tx_stderr.send(event).is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // --- idle watchdog thread ---
        let idle_timeout_secs = params.timeout_secs;
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(5));

                if process_done.load(Ordering::Relaxed) {
                    break;
                }

                let last = last_active.load(Ordering::Relaxed);
                let now_secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if now_secs.saturating_sub(last) > idle_timeout_secs {
                    let _ = tx.send(StreamEvent {
                        text: String::new(),
                        is_done: true,
                        error: Some(format!(
                            "Timeout: no CLI output for {} seconds (idle timeout)",
                            idle_timeout_secs
                        )),
                        msg_type: Some("timeout".to_string()),
                        session_id: None,
                        content_blocks: None,
                    });
                    break;
                }
            }
        });

        Ok(rx)
    }
}
