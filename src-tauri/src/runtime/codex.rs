//! OpenAI Codex CLI runtime implementation.
//!
//! Wraps the Codex CLI (`codex`) as an AgentRuntime.
//! Uses stdin/stdout streaming for interaction.

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
// CodexRuntime
// ===========================================================================

/// OpenAI Codex CLI runtime implementation.
#[derive(Default)]
pub struct CodexRuntime;

impl CodexRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl AgentRuntime for CodexRuntime {
    fn id(&self) -> &str {
        "codex"
    }

    fn name(&self) -> &str {
        "OpenAI Codex"
    }

    fn runtime_category(&self) -> &str {
        "cli"
    }

    fn typed_runtime_type(&self) -> RuntimeType {
        RuntimeType::Codex
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Streaming,
            AgentCapability::Sessions,
            AgentCapability::ToolUse,
        ]
    }

    fn install_hint(&self) -> String {
        "npm install -g @openai/codex".to_string()
    }

    fn binary_name(&self) -> &str {
        "codex"
    }

    fn detect(&self) -> Result<Option<(String, String)>, String> {
        let path = RuntimeDetector::find_command("codex");
        match path {
            Some(p) => {
                let version =
                    RuntimeDetector::get_version("codex").unwrap_or_else(|| "unknown".to_string());
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
                "Codex CLI not found or not healthy. Please install: npm install -g @openai/codex"
                    .to_string(),
            );
        }

        // Build CLI arguments
        // codex CLI accepts messages via stdin and streams responses
        let mut args: Vec<String> = vec![
            "--quiet".to_string(), // non-interactive mode
        ];

        if let Some(ref ws) = params.workspace {
            if !ws.is_empty() {
                args.push("--cwd".to_string());
                args.push(ws.clone());
            }
        }

        // Spawn CLI process
        let mut cmd = Command::new("codex");
        if let Some(ref ws) = params.workspace {
            if !ws.is_empty() {
                cmd.current_dir(ws);
            }
        }
        let mut child = cmd
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn Codex CLI: {}", e))?;

        // Write message to stdin
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(params.message.as_bytes());
            let _ = stdin.write_all(b"\n");
            drop(stdin); // Close stdin to signal EOF
        }

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

                        // Try to parse as JSON (codex may output JSON)
                        match serde_json::from_str::<serde_json::Value>(trimmed) {
                            Ok(json_obj) => {
                                let msg_type = json_obj
                                    .get("type")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();

                                let is_result = msg_type == "result" || msg_type == "done";
                                if is_result {
                                    got_result = true;
                                }

                                let text_content = json_obj
                                    .get("content")
                                    .and_then(|c| c.as_str())
                                    .unwrap_or(trimmed)
                                    .to_string();

                                let event = StreamEvent {
                                    text: text_content,
                                    is_done: is_result,
                                    error: None,
                                    msg_type: Some(msg_type),
                                    session_id: None,
                                    content_blocks: None,
                                };

                                if tx_stdout.send(event).is_err() {
                                    break;
                                }
                            }
                            Err(_) => {
                                // Non-JSON: emit as raw text
                                let event = StreamEvent {
                                    text: trimmed.to_string(),
                                    is_done: false,
                                    error: None,
                                    msg_type: Some("raw".to_string()),
                                    session_id: None,
                                    content_blocks: None,
                                };
                                if tx_stdout.send(event).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            process_done_stdout.store(true, Ordering::Relaxed);

            if !got_result {
                let _ = tx_stdout.send(StreamEvent {
                    text: String::new(),
                    is_done: true,
                    error: None,
                    msg_type: Some("process_exit".to_string()),
                    session_id: None,
                    content_blocks: None,
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
                            last_active_stderr.store(
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs(),
                                Ordering::Relaxed,
                            );
                            log::warn!("[CodexRuntime] stderr: {}", trimmed);
                            let event = StreamEvent {
                                text: String::new(),
                                is_done: false,
                                error: Some(format!("CLI stderr: {}", trimmed)),
                                msg_type: Some("stderr".to_string()),
                                session_id: None,
                                content_blocks: None,
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
