//! Claude Code CLI runtime implementation.
//!
//! Wraps the Claude Code CLI (`claude`) as an AgentRuntime.
//! Uses Control Protocol mode (`--print --input-format stream-json --output-format stream-json`)
//! for persistent bidirectional JSON communication over stdin/stdout.
//!
//! Each agent gets a dedicated long-running `claude` process. Messages are sent
//! via stdin as JSON, responses are received via stdout as JSONL — parsed into
//! StreamEvents and forwarded through an mpsc channel.

use super::{
    AgentCapability, AgentRuntime, AgentRuntimeInfo, AgentRuntimeStatus, ExecuteParams,
    RuntimeDetector, RuntimeType, StreamEvent,
};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ===========================================================================
// ProcessHandle — a persistent claude process for one agent
// ===========================================================================

/// Manages a single long-running `claude` CLI process for a specific agent.
struct ProcessHandle {
    /// The child process
    child: Child,
    /// stdin writer for sending JSON messages (None after shutdown)
    stdin_writer: Option<BufWriter<ChildStdin>>,
    /// The current request's output channel sender.
    /// When a new execute() call comes in, the sender is swapped.
    current_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<StreamEvent>>>>,
    /// Session ID extracted from the first response (used for crash recovery)
    session_id: Option<String>,
    /// Last activity timestamp (epoch seconds) for idle timeout
    last_active: Arc<AtomicU64>,
    /// Whether the stdout reader thread is still active
    reader_alive: Arc<AtomicBool>,
    /// Workspace directory this process was started with
    workspace: Option<String>,
}

impl ProcessHandle {
    /// Check if the underlying process is still alive.
    fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Send a user message to the process via stdin.
    fn send_user_message(&mut self, message: &str) -> Result<(), String> {
        let writer = self
            .stdin_writer
            .as_mut()
            .ok_or_else(|| "stdin writer not available (process shut down)".to_string())?;
        let json_msg = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": [{"type": "text", "text": message}]
            }
        });
        let line = format!("{}\n", json_msg);
        writer
            .write_all(line.as_bytes())
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
        writer
            .flush()
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;
        Ok(())
    }

    /// Gracefully shut down the process.
    fn shutdown(&mut self) {
        // Close stdin to signal the process to exit
        if let Some(mut writer) = self.stdin_writer.take() {
            let _ = writer.flush();
            drop(writer);
        }
        // Give it a moment to exit gracefully
        std::thread::sleep(Duration::from_millis(100));
        // Force kill if still alive
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// ===========================================================================
// ClaudeCodeRuntime
// ===========================================================================

/// Claude Code CLI runtime with persistent process pool.
///
/// Each `agent_id` maps to a dedicated long-running `claude` process.
/// Messages are sent via stdin JSON, responses arrive via stdout JSONL.
pub struct ClaudeCodeRuntime {
    processes: Arc<Mutex<HashMap<String, ProcessHandle>>>,
}

impl ClaudeCodeRuntime {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get an existing process or spawn a new one for the given agent.
    fn get_or_spawn(
        &self,
        agent_id: &str,
        workspace: Option<&str>,
        system_prompt: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<(), String> {
        let mut processes = self.processes.lock().map_err(|e| e.to_string())?;

        // Check if we have a live process for this agent
        if let Some(handle) = processes.get_mut(agent_id) {
            if handle.is_alive() && handle.reader_alive.load(Ordering::Relaxed) {
                log::info!(
                    "[ClaudeCodeRuntime] Reusing existing process for agent {}",
                    agent_id
                );
                return Ok(());
            }
            // Process is dead, remove it (Drop will clean up)
            log::info!(
                "[ClaudeCodeRuntime] Dead process detected for agent {}, respawning",
                agent_id
            );
            let old_session = handle.session_id.clone();
            processes.remove(agent_id);
            // Fall through to spawn new process, possibly resuming session
            drop(processes);
            return self.spawn_process(
                agent_id,
                workspace,
                system_prompt,
                session_id.or(old_session.as_deref()),
            );
        }

        // No existing process, spawn new one
        drop(processes);
        self.spawn_process(agent_id, workspace, system_prompt, session_id)
    }

    /// Spawn a new claude process for the given agent.
    fn spawn_process(
        &self,
        agent_id: &str,
        workspace: Option<&str>,
        system_prompt: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<(), String> {
        // Build CLI arguments
        let mut args: Vec<String> = vec![
            "--print".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--input-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
            "--dangerously-skip-permissions".to_string(),
        ];

        if let Some(sid) = session_id {
            args.push("--resume".to_string());
            args.push(sid.to_string());
        }

        if let Some(sp) = system_prompt {
            args.push("--append-system-prompt".to_string());
            args.push(sp.to_string());
        }

        if let Some(ws) = workspace {
            if !ws.is_empty() {
                args.push("--add-dir".to_string());
                args.push(ws.to_string());
            }
        }

        log::info!(
            "[ClaudeCodeRuntime] Spawning persistent process for agent {} with args: {:?}",
            agent_id,
            args.iter().take(10).collect::<Vec<_>>()
        );

        // Spawn CLI process
        let mut cmd = Command::new("claude");
        if let Some(ws) = workspace {
            if !ws.is_empty() {
                cmd.current_dir(ws);
            }
        }

        let mut child = cmd
            .args(&args)
            .stdin(Stdio::piped()) // stdin is piped for writing
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
        let stdin_handle = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to get stdin handle".to_string())?;

        let current_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<StreamEvent>>>> =
            Arc::new(Mutex::new(None));
        let last_active = Arc::new(AtomicU64::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ));
        let reader_alive = Arc::new(AtomicBool::new(true));

        // Session ID shared between stdout reader and ProcessHandle
        let shared_session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        // --- stdout reader thread (persistent) ---
        let tx_stdout = current_sender.clone();
        let last_active_stdout = last_active.clone();
        let reader_alive_stdout = reader_alive.clone();
        let session_id_stdout = shared_session_id.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout_handle);

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

                        // Parse JSON (same format as --print mode)
                        match serde_json::from_str::<serde_json::Value>(trimmed) {
                            Ok(json_obj) => {
                                let msg_type = json_obj
                                    .get("type")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();

                                // Extract and cache session_id from any event
                                if let Some(sid) = json_obj
                                    .get("session_id")
                                    .and_then(|s| s.as_str())
                                    .map(|s| s.to_string())
                                {
                                    let mut sess = session_id_stdout.lock().unwrap();
                                    if sess.is_none() {
                                        *sess = Some(sid);
                                    }
                                }

                                let is_result = msg_type == "result";

                                // Extract text content
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

                                fn extract_structured_blocks(
                                    val: &serde_json::Value,
                                ) -> Option<serde_json::Value> {
                                    if val.is_array() {
                                        let blocks: Vec<serde_json::Value> = val
                                            .as_array()
                                            .unwrap()
                                            .iter()
                                            .filter(|item| {
                                                let btype = item
                                                    .get("type")
                                                    .and_then(|t| t.as_str())
                                                    .unwrap_or("");
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

                                let content_val =
                                    if msg_type == "assistant" || msg_type == "user" {
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

                                let content_blocks = content_val.and_then(extract_structured_blocks);

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

                                let response_session_id = json_obj
                                    .get("session_id")
                                    .and_then(|s| s.as_str())
                                    .map(|s| s.to_string());

                                let event = StreamEvent {
                                    text,
                                    is_done: is_result,
                                    error,
                                    msg_type: Some(msg_type),
                                    session_id: response_session_id,
                                    content_blocks,
                                };

                                // Send to the current request's channel
                                if let Ok(sender) = tx_stdout.lock() {
                                    if let Some(ref tx) = *sender {
                                        if tx.send(event).is_err() {
                                            // Channel closed, that's fine
                                        }
                                    }
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
                                if let Ok(sender) = tx_stdout.lock() {
                                    if let Some(ref tx) = *sender {
                                        let _ = tx.send(event);
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => break, // EOF — process exited
                }
            }

            reader_alive_stdout.store(false, Ordering::Relaxed);
            log::info!("[ClaudeCodeRuntime] stdout reader thread exited");
        });

        // --- stderr reader thread (persistent) ---
        let tx_stderr = current_sender.clone();
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
                            log::warn!("[ClaudeCodeRuntime] stderr: {}", trimmed);
                            let event = StreamEvent {
                                text: String::new(),
                                is_done: false,
                                error: Some(format!("CLI stderr: {}", trimmed)),
                                content_blocks: None,
                                msg_type: Some("stderr".to_string()),
                                session_id: None,
                            };
                            if let Ok(sender) = tx_stderr.lock() {
                                if let Some(ref tx) = *sender {
                                    let _ = tx.send(event);
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Store the process handle
        let handle = ProcessHandle {
            child,
            stdin_writer: Some(BufWriter::new(stdin_handle)),
            current_sender,
            session_id: None, // Will be populated by stdout reader
            last_active,
            reader_alive,
            workspace: workspace.map(|s| s.to_string()),
        };

        let mut processes = self.processes.lock().map_err(|e| e.to_string())?;
        processes.insert(agent_id.to_string(), handle);

        log::info!(
            "[ClaudeCodeRuntime] Spawned persistent process for agent {}",
            agent_id
        );
        Ok(())
    }

    /// Kill the process for a specific agent.
    pub fn kill_process(&self, agent_id: &str) -> Result<(), String> {
        let mut processes = self.processes.lock().map_err(|e| e.to_string())?;
        if processes.remove(agent_id).is_some() {
            log::info!("[ClaudeCodeRuntime] Killed process for agent {}", agent_id);
        }
        Ok(())
    }

    /// Kill all processes.
    pub fn cleanup_all(&self) {
        let mut processes = self.processes.lock().unwrap();
        let count = processes.len();
        processes.clear(); // Drop will call shutdown on each
        if count > 0 {
            log::info!(
                "[ClaudeCodeRuntime] Cleaned up {} processes",
                count
            );
        }
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
        // Pre-flight: verify CLI is available
        if !self.is_ready() {
            return Err(
                "Claude Code CLI not found or not healthy. Please install: npm install -g @anthropic-ai/claude-code"
                    .to_string(),
            );
        }

        log::info!(
            "[ClaudeCodeRuntime] execute() agent_id={}, session_id={:?}",
            params.agent_id,
            params.session_id
        );

        // Get or spawn a persistent process for this agent
        self.get_or_spawn(
            &params.agent_id,
            params.workspace.as_deref(),
            params.system_prompt.as_deref(),
            params.session_id.as_deref(),
        )?;

        // Create a new channel for this request
        let (tx, rx) = std::sync::mpsc::channel();

        // Swap the current_sender in the ProcessHandle
        {
            let mut processes = self.processes.lock().map_err(|e| e.to_string())?;
            if let Some(handle) = processes.get_mut(&params.agent_id) {
                // Replace the sender — this routes stdout output to our new channel
                {
                    let mut sender = handle.current_sender.lock().unwrap();
                    *sender = Some(tx.clone());
                }

                // Send the user message via stdin
                handle.send_user_message(&params.message)?;
            } else {
                return Err(format!(
                    "No process found for agent {} after spawn",
                    params.agent_id
                ));
            }
        }

        // Spawn idle watchdog for this specific request
        let tx_watchdog = tx;
        let idle_timeout_secs = params.timeout_secs;
        let processes = self.processes.clone();
        let agent_id = params.agent_id.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(5));

                let last_active = {
                    let Ok(proc_map) = processes.lock() else { break };
                    let Some(handle) = proc_map.get(&agent_id) else { break };
                    handle.last_active.load(Ordering::Relaxed)
                };

                let now_secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if now_secs.saturating_sub(last_active) > idle_timeout_secs {
                    let _ = tx_watchdog.send(StreamEvent {
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

impl Default for ClaudeCodeRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ClaudeCodeRuntime {
    fn drop(&mut self) {
        self.cleanup_all();
    }
}
