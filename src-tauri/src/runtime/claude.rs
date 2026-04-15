//! Claude Code CLI runtime implementation.
//!
//! Wraps the Claude Code CLI (`claude`) as an AgentRuntime.
//!
//! Two execution modes based on caller context:
//!
//! - **Thread mode** (`session_id = Some`): Persistent process with stdin/stdout
//!   JSON streaming. The process is reused across messages for the same agent,
//!   avoiding cold-start overhead. Idle processes are killed after a timeout
//!   and can be restored via `--resume`.
//!
//! - **Channel mode** (`session_id = None`): One-shot spawn per request.
//!   Channel reconstructs context (sliding window + summary) each time, so a
//!   persistent process would conflict with the orchestration layer. Each
//!   request gets a fresh process with the full reconstructed system_prompt.

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

/// Maximum number of concurrent persistent claude processes.
const MAX_CONCURRENT_PROCESSES: usize = 5;

/// Idle timeout in seconds before a persistent process is killed.
const IDLE_TIMEOUT_SECS: u64 = 300; // 5 minutes

// ===========================================================================
// ProcessHandle — a persistent claude process for one agent (Thread mode)
// ===========================================================================

/// Manages a single persistent `claude` CLI process for Thread-mode conversations.
struct ProcessHandle {
    /// The child process
    child: Child,
    /// stdin writer for sending JSON messages (None after shutdown)
    stdin_writer: Option<BufWriter<ChildStdin>>,
    /// The current request's output channel sender.
    current_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<StreamEvent>>>>,
    /// Session ID extracted from the first response (used for --resume after eviction)
    session_id: Option<String>,
    /// Last activity timestamp (epoch seconds) for idle timeout
    last_active: Arc<AtomicU64>,
    /// Whether the stdout reader thread is still active
    reader_alive: Arc<AtomicBool>,
    /// Workspace directory this process was started with
    workspace: Option<String>,
    /// Monotonic timestamp (Instant) for LRU eviction ordering
    last_used_epoch: u64,
}

impl ProcessHandle {
    /// Check if the underlying process is still alive.
    fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Send a user message to the process via stdin (Control Protocol).
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
        if let Some(mut writer) = self.stdin_writer.take() {
            let _ = writer.flush();
            drop(writer);
        }
        std::thread::sleep(Duration::from_millis(100));
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

/// Claude Code CLI runtime with dual execution strategy.
///
/// - Thread calls (session_id present): persistent process pool with LRU eviction
/// - Channel calls (session_id absent): one-shot spawn per request
pub struct ClaudeCodeRuntime {
    /// Persistent process pool for Thread-mode conversations.
    /// Keyed by agent_id (from ExecuteParams.agent_id).
    processes: Arc<Mutex<HashMap<String, ProcessHandle>>>,
    /// Monotonic counter for LRU eviction ordering.
    epoch_counter: Arc<AtomicU64>,
}

impl ClaudeCodeRuntime {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            epoch_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get current epoch counter and increment.
    fn next_epoch(&self) -> u64 {
        self.epoch_counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Evict the least recently used process to make room.
    /// Must be called with processes lock held.
    fn evict_lru(processes: &mut HashMap<String, ProcessHandle>) {
        if processes.is_empty() {
            return;
        }
        let lru_key = processes
            .iter()
            .min_by_key(|(_, h)| h.last_used_epoch)
            .map(|(k, _)| k.clone());
        if let Some(key) = lru_key {
            log::info!("[ClaudeCodeRuntime] LRU evicting process for agent {}", key);
            processes.remove(&key); // Drop → shutdown
        }
    }

    /// Kill idle processes that exceeded the timeout.
    fn cleanup_idle(&self) {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut processes = match self.processes.lock() {
            Ok(p) => p,
            Err(_) => return,
        };

        let to_remove: Vec<String> = processes
            .iter()
            .filter_map(|(key, handle)| {
                let last = handle.last_active.load(Ordering::Relaxed);
                if now_secs.saturating_sub(last) > IDLE_TIMEOUT_SECS {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in to_remove {
            log::info!(
                "[ClaudeCodeRuntime] Idle timeout, killing process for agent {}",
                key
            );
            processes.remove(&key); // Drop → shutdown
        }
    }

    /// Get existing persistent process or spawn new one (Thread mode only).
    fn get_or_spawn_thread(
        &self,
        agent_id: &str,
        workspace: Option<&str>,
        system_prompt: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<(), String> {
        // First, clean up any idle processes
        self.cleanup_idle();

        let mut processes = self.processes.lock().map_err(|e| e.to_string())?;
        let epoch = self.next_epoch();

        // Check if we have a live process for this agent
        if let Some(handle) = processes.get_mut(agent_id) {
            if handle.is_alive() && handle.reader_alive.load(Ordering::Relaxed) {
                handle.last_used_epoch = epoch;
                log::info!(
                    "[ClaudeCodeRuntime] Reusing persistent process for agent {}",
                    agent_id
                );
                return Ok(());
            }
            // Dead process — extract session_id for resume
            let old_session = handle.session_id.clone();
            processes.remove(agent_id);
            drop(processes);
            return self.spawn_persistent(
                agent_id,
                workspace,
                system_prompt,
                session_id.or(old_session.as_deref()),
                epoch,
            );
        }

        // No existing process, spawn new
        drop(processes);
        self.spawn_persistent(agent_id, workspace, system_prompt, session_id, epoch)
    }

    /// Spawn a persistent claude process (Thread mode).
    fn spawn_persistent(
        &self,
        agent_id: &str,
        workspace: Option<&str>,
        system_prompt: Option<&str>,
        session_id: Option<&str>,
        epoch: u64,
    ) -> Result<(), String> {
        let mut processes = self.processes.lock().map_err(|e| e.to_string())?;

        // Enforce max concurrent limit via LRU eviction
        while processes.len() >= MAX_CONCURRENT_PROCESSES {
            Self::evict_lru(&mut processes);
        }

        let args = build_cli_args(system_prompt, session_id, workspace, true);

        log::info!(
            "[ClaudeCodeRuntime] Spawning persistent process for agent {} (session_id={:?})",
            agent_id,
            session_id
        );

        let (child, stdin_handle, stdout_handle, stderr_handle) =
            spawn_cli_process(&args, workspace)?;

        let current_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<StreamEvent>>>> =
            Arc::new(Mutex::new(None));
        let last_active = Arc::new(AtomicU64::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ));
        let reader_alive = Arc::new(AtomicBool::new(true));
        let shared_session_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        // Spawn stdout/stderr reader threads
        spawn_stdout_reader(
            stdout_handle,
            current_sender.clone(),
            last_active.clone(),
            reader_alive.clone(),
            shared_session_id.clone(),
        );
        spawn_stderr_reader(stderr_handle, current_sender.clone(), last_active.clone());

        let handle = ProcessHandle {
            child,
            stdin_writer: Some(BufWriter::new(stdin_handle)),
            current_sender,
            session_id: None,
            last_active,
            reader_alive,
            workspace: workspace.map(|s| s.to_string()),
            last_used_epoch: epoch,
        };

        processes.insert(agent_id.to_string(), handle);
        Ok(())
    }

    /// Execute in one-shot mode (Channel mode).
    /// Spawns a fresh process for this request only, no reuse.
    fn execute_oneshot(
        &self,
        params: &ExecuteParams,
    ) -> Result<std::sync::mpsc::Receiver<StreamEvent>, String> {
        let args = build_cli_args(
            params.system_prompt.as_deref(),
            params.session_id.as_deref(),
            params.workspace.as_deref(),
            false, // no --input-format (one-shot doesn't need stdin JSON)
        );

        // For one-shot mode, append the message as final positional arg
        let mut full_args = args;
        full_args.push("--".to_string());
        full_args.push(params.message.clone());

        log::info!(
            "[ClaudeCodeRuntime] One-shot spawn for agent {} (channel mode)",
            params.agent_id
        );

        let mut cmd = Command::new("claude");
        if let Some(ref ws) = params.workspace {
            if !ws.is_empty() {
                cmd.current_dir(ws);
            }
        }

        let mut child = cmd
            .args(&full_args)
            .stdin(Stdio::null()) // no stdin for one-shot
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
        let last_active = Arc::new(AtomicU64::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ));
        let process_done = Arc::new(AtomicBool::new(false));

        // stdout reader thread
        let tx_stdout = tx.clone();
        let last_active_stdout = last_active.clone();
        let process_done_stdout = process_done.clone();
        let child_id = child.id();
        std::thread::spawn(move || {
            read_stdout_to_channel(
                stdout_handle,
                tx_stdout,
                last_active_stdout,
                Some(process_done_stdout),
            );
            log::info!(
                "[ClaudeCodeRuntime] One-shot process {} stdout reader exited",
                child_id
            );
        });

        // stderr reader thread
        let tx_stderr = tx.clone();
        let last_active_stderr = last_active.clone();
        std::thread::spawn(move || {
            read_stderr_to_channel(stderr_handle, tx_stderr, last_active_stderr);
        });

        // idle watchdog thread
        let tx_watchdog = tx;
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
                    let _ = tx_watchdog.send(StreamEvent {
                        text: String::new(),
                        is_done: true,
                        error: Some(format!(
                            "Timeout: no CLI output for {} seconds",
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

        // Reap the child process in background to avoid zombies
        std::thread::spawn(move || {
            let _ = child.wait();
        });

        Ok(rx)
    }

    /// Execute using persistent process (Thread mode).
    fn execute_persistent(
        &self,
        params: &ExecuteParams,
    ) -> Result<std::sync::mpsc::Receiver<StreamEvent>, String> {
        // Get or spawn a persistent process
        self.get_or_spawn_thread(
            &params.agent_id,
            params.workspace.as_deref(),
            params.system_prompt.as_deref(),
            params.session_id.as_deref(),
        )?;

        let (tx, rx) = std::sync::mpsc::channel();

        // Swap the current_sender and send message
        {
            let mut processes = self.processes.lock().map_err(|e| e.to_string())?;
            if let Some(handle) = processes.get_mut(&params.agent_id) {
                {
                    let mut sender = handle.current_sender.lock().unwrap();
                    *sender = Some(tx.clone());
                }
                handle.send_user_message(&params.message)?;
            } else {
                return Err(format!(
                    "No process found for agent {} after spawn",
                    params.agent_id
                ));
            }
        }

        // Idle watchdog for this request
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
                            "Timeout: no CLI output for {} seconds",
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

    /// Kill the persistent process for a specific agent.
    pub fn kill_process(&self, agent_id: &str) -> Result<(), String> {
        let mut processes = self.processes.lock().map_err(|e| e.to_string())?;
        if processes.remove(agent_id).is_some() {
            log::info!("[ClaudeCodeRuntime] Killed process for agent {}", agent_id);
        }
        Ok(())
    }

    /// Kill all persistent processes.
    pub fn cleanup_all(&self) {
        let mut processes = self.processes.lock().unwrap();
        let count = processes.len();
        processes.clear();
        if count > 0 {
            log::info!("[ClaudeCodeRuntime] Cleaned up {} processes", count);
        }
    }
}

// ===========================================================================
// Shared helper functions
// ===========================================================================

/// Build CLI argument list for claude invocation.
fn build_cli_args(
    system_prompt: Option<&str>,
    session_id: Option<&str>,
    workspace: Option<&str>,
    with_input_format: bool,
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "--print".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
        "--dangerously-skip-permissions".to_string(),
    ];

    // Only add --input-format for persistent (Thread) mode
    if with_input_format {
        args.push("--input-format".to_string());
        args.push("stream-json".to_string());
    }

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

    args
}

/// Spawn a claude CLI process with piped stdin/stdout/stderr.
fn spawn_cli_process(
    args: &[String],
    workspace: Option<&str>,
) -> Result<(Child, ChildStdin, std::process::ChildStdout, std::process::ChildStderr), String> {
    let mut cmd = Command::new("claude");
    if let Some(ws) = workspace {
        if !ws.is_empty() {
            cmd.current_dir(ws);
        }
    }

    let mut child = cmd
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn Claude CLI: {}", e))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to get stdin handle".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to get stdout handle".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to get stderr handle".to_string())?;

    Ok((child, stdin, stdout, stderr))
}

/// Persistent stdout reader: parses JSONL lines into StreamEvents and sends
/// them through the current_sender channel (which gets swapped per request).
fn spawn_stdout_reader(
    stdout: std::process::ChildStdout,
    current_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<StreamEvent>>>>,
    last_active: Arc<AtomicU64>,
    reader_alive: Arc<AtomicBool>,
    shared_session_id: Arc<Mutex<Option<String>>>,
) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            match line {
                Ok(text) => {
                    last_active.store(
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

                    match serde_json::from_str::<serde_json::Value>(trimmed) {
                        Ok(json_obj) => {
                            let event = parse_stream_event(&json_obj, trimmed);

                            // Cache session_id from any event
                            if let Some(sid) = json_obj
                                .get("session_id")
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_string())
                            {
                                let mut sess = shared_session_id.lock().unwrap();
                                if sess.is_none() {
                                    *sess = Some(sid);
                                }
                            }

                            send_to_current_sender(&current_sender, event);
                        }
                        Err(_) => {
                            let event = StreamEvent {
                                text: trimmed.to_string(),
                                is_done: false,
                                error: None,
                                content_blocks: None,
                                msg_type: Some("raw".to_string()),
                                session_id: None,
                            };
                            send_to_current_sender(&current_sender, event);
                        }
                    }
                }
                Err(_) => break,
            }
        }

        reader_alive.store(false, Ordering::Relaxed);
        log::info!("[ClaudeCodeRuntime] Persistent stdout reader exited");
    });
}

/// Stderr reader thread: logs warnings and forwards errors.
fn spawn_stderr_reader(
    stderr: std::process::ChildStderr,
    current_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<StreamEvent>>>>,
    last_active: Arc<AtomicU64>,
) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        last_active.store(
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
                        send_to_current_sender(&current_sender, event);
                    }
                }
                Err(_) => break,
            }
        }
    });
}

/// One-shot stdout reader: reads from a process that will exit on its own.
fn read_stdout_to_channel(
    stdout: std::process::ChildStdout,
    tx: std::sync::mpsc::Sender<StreamEvent>,
    last_active: Arc<AtomicU64>,
    process_done: Option<Arc<AtomicBool>>,
) {
    let reader = BufReader::new(stdout);
    let mut got_result = false;

    for line in reader.lines() {
        match line {
            Ok(text) => {
                last_active.store(
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

                match serde_json::from_str::<serde_json::Value>(trimmed) {
                    Ok(json_obj) => {
                        let event = parse_stream_event(&json_obj, trimmed);
                        if event.is_done {
                            got_result = true;
                        }
                        if tx.send(event).is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        let event = StreamEvent {
                            text: trimmed.to_string(),
                            is_done: false,
                            error: None,
                            content_blocks: None,
                            msg_type: Some("raw".to_string()),
                            session_id: None,
                        };
                        if tx.send(event).is_err() {
                            break;
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }

    if let Some(done) = process_done {
        done.store(true, Ordering::Relaxed);
    }

    if !got_result {
        let _ = tx.send(StreamEvent {
            text: String::new(),
            is_done: true,
            error: None,
            content_blocks: None,
            msg_type: Some("process_exit".to_string()),
            session_id: None,
        });
    }
}

/// One-shot stderr reader.
fn read_stderr_to_channel(
    stderr: std::process::ChildStderr,
    tx: std::sync::mpsc::Sender<StreamEvent>,
    last_active: Arc<AtomicU64>,
) {
    let reader = BufReader::new(stderr);
    for line in reader.lines() {
        match line {
            Ok(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    last_active.store(
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
                    if tx.send(event).is_err() {
                        break;
                    }
                }
            }
            Err(_) => break,
        }
    }
}

/// Parse a JSONL line from claude stdout into a StreamEvent.
fn parse_stream_event(json_obj: &serde_json::Value, raw_line: &str) -> StreamEvent {
    let msg_type = json_obj
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown")
        .to_string();

    let is_result = msg_type == "result";

    fn extract_text_content(val: &serde_json::Value) -> String {
        if let Some(s) = val.as_str() {
            s.to_string()
        } else if let Some(arr) = val.as_array() {
            arr.iter()
                .filter_map(|item| {
                    if item.get("type").and_then(|t| t.as_str()) == Some("text") {
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

    fn extract_structured_blocks(val: &serde_json::Value) -> Option<serde_json::Value> {
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

    let content_val = if msg_type == "assistant" || msg_type == "user" {
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
                raw_line.to_string()
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

    let session_id = json_obj
        .get("session_id")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    StreamEvent {
        text,
        is_done: is_result,
        error,
        msg_type: Some(msg_type),
        session_id,
        content_blocks,
    }
}

/// Send an event through the current_sender channel.
fn send_to_current_sender(
    sender: &Arc<Mutex<Option<std::sync::mpsc::Sender<StreamEvent>>>>,
    event: StreamEvent,
) {
    if let Ok(guard) = sender.lock() {
        if let Some(ref tx) = *guard {
            let _ = tx.send(event);
        }
    }
}

// ===========================================================================
// AgentRuntime trait implementation
// ===========================================================================

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
        if !self.is_ready() {
            return Err(
                "Claude Code CLI not found or not healthy. Please install: npm install -g @anthropic-ai/claude-code"
                    .to_string(),
            );
        }

        // Route based on persistent flag:
        //   persistent = true  → Thread mode: persistent process (reuse or spawn)
        //   persistent = false → Channel mode: one-shot (fresh process each time)
        if params.persistent {
            log::info!(
                "[ClaudeCodeRuntime] Thread mode: agent={}, session_id={:?}",
                params.agent_id,
                params.session_id
            );
            self.execute_persistent(&params)
        } else {
            log::info!(
                "[ClaudeCodeRuntime] Channel mode: agent={}, one-shot spawn",
                params.agent_id
            );
            self.execute_oneshot(&params)
        }
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
