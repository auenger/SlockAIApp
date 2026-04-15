//! Adapter-backed A2A Server handlers.
//!
//! Extends the A2A Server skeleton with handlers that delegate task execution
//! to a CLI adapter (Claude Code or Codex). Provides:
//! - Task CRUD with real execution via adapter
//! - SendMessage that triggers CLI execution
//! - StreamMessage with SSE-style event forwarding
//! - AgentCard generation from adapter metadata
//!
//! Also provides a listener that can serve the A2A HTTP protocol over
//! TCP or Unix sockets.

use super::cli_adapter::{AdapterConfig, AdapterState, CliA2AAdapter};
use crate::runtime::StreamEvent;
use crate::runtime::a2a::bridge::stream_event_to_a2a_message;
use crate::runtime::a2a::server::{A2AServer, A2AServerConfig};
use crate::runtime::a2a::types::*;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// ===========================================================================
// AdapterServer — wraps A2AServer with an adapter backend
// ===========================================================================

/// An A2A Server backed by a CLI adapter.
///
/// This struct combines:
/// - An `A2AServer` for JSON-RPC dispatch
/// - A shared `CliA2AAdapter` for actual CLI execution
/// - Shared state for tracking tasks
/// - Buffered messages per task for query responses
pub struct AdapterServer {
    /// The inner A2A server.
    server: A2AServer,
    /// The CLI adapter that handles execution (shared via Arc for closures).
    adapter: Arc<dyn CliA2AAdapter>,
    /// Shared adapter state for task tracking.
    state: Arc<Mutex<AdapterState>>,
    /// Buffered task messages (task_id -> Vec<Message>).
    task_messages: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    /// Active streaming receivers (task_id -> Receiver<StreamEvent>).
    active_streams: Arc<Mutex<HashMap<String, std::sync::mpsc::Receiver<StreamEvent>>>>,
}

impl AdapterServer {
    /// Create a new adapter server with the given adapter and agent card.
    pub fn new(adapter: Box<dyn CliA2AAdapter>, agent_card: AgentCard) -> Self {
        let state = AdapterState::shared();
        let server = A2AServer::new(A2AServerConfig::new(agent_card));

        Self {
            server,
            adapter: Arc::from(adapter),
            state,
            task_messages: Arc::new(Mutex::new(HashMap::new())),
            active_streams: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create with a custom server config.
    pub fn with_config(
        adapter: Box<dyn CliA2AAdapter>,
        config: A2AServerConfig,
    ) -> Self {
        let state = AdapterState::shared();
        let server = A2AServer::new(config);

        Self {
            server,
            adapter: Arc::from(adapter),
            state,
            task_messages: Arc::new(Mutex::new(HashMap::new())),
            active_streams: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the server configuration.
    pub fn config(&self) -> &A2AServerConfig {
        self.server.config()
    }

    /// Get the agent card.
    pub fn agent_card(&self) -> &AgentCard {
        self.server.agent_card()
    }

    /// Register all A2A handlers backed by the adapter.
    ///
    /// This sets up:
    /// - `sendMessage` — create or continue a task, execute via adapter
    /// - `streamMessage` — execute and return SSE-compatible events
    /// - `getTask` — get task status from adapter state
    /// - `listTasks` — list all tracked tasks
    /// - `cancelTask` — cancel a running task
    pub fn register_adapter_handlers(&self, default_config: AdapterConfig) {
        let state = self.state.clone();
        let task_messages = self.task_messages.clone();
        let active_streams = self.active_streams.clone();

        // --- sendMessage handler ---
        let adapter_send = self.adapter.clone();
        let state_send = state.clone();
        let task_messages_send = task_messages.clone();
        let active_streams_send = active_streams.clone();
        let config_send = default_config.clone();

        self.server.register_handler("sendMessage", move |params| {
            let req: SendMessageRequest = serde_json::from_value(params)
                .map_err(|e| A2AError::invalid_params(format!("Invalid sendMessage params: {}", e)))?;

            let task_id = req.task.id.clone();

            // Extract user message text from the task
            let user_text = req.task.messages.iter()
                .filter_map(|m| {
                    if m.role == MessageRole::User {
                        m.parts.iter().filter_map(|p| match p {
                            Part::Text { text } => Some(text.as_str()),
                            _ => None,
                        }).collect::<Vec<_>>().into_iter().next()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            if user_text.is_empty() {
                return Err(A2AError::invalid_params("No user message text found in task"));
            }

            // Store messages
            {
                let mut msgs = task_messages_send.lock().unwrap();
                msgs.entry(task_id.clone())
                    .or_insert_with(Vec::new)
                    .extend(req.task.messages.clone());
            }

            // Execute the task via adapter
            match adapter_send.execute_task(&task_id, &user_text, &config_send) {
                Ok(rx) => {
                    // Store the receiver for potential streaming
                    {
                        let mut streams = active_streams_send.lock().unwrap();
                        streams.insert(task_id.clone(), rx);
                    }

                    // Build response task
                    let response_task = {
                        let s = state_send.lock().unwrap();
                        s.build_task(&task_id).unwrap_or_else(|| Task {
                            id: task_id.clone(),
                            status: TaskStatus::Working,
                            session_id: None,
                            messages: Vec::new(),
                            artifacts: Vec::new(),
                            metadata: None,
                        })
                    };

                    Ok(serde_json::to_value(SendMessageResponse {
                        task: response_task,
                    }).unwrap_or_default())
                }
                Err(e) => Err(e),
            }
        });

        // --- streamMessage handler ---
        let state_stream = state.clone();
        let active_streams_stream = active_streams.clone();

        self.server.register_handler("streamMessage", move |params| {
            let req: SendMessageRequest = serde_json::from_value(params)
                .map_err(|e| A2AError::invalid_params(format!("Invalid streamMessage params: {}", e)))?;

            let task_id = req.task.id.clone();

            // Check if there's an active stream for this task
            let events_json = {
                let mut streams = active_streams_stream.lock().unwrap();
                if let Some(rx) = streams.remove(&task_id) {
                    // Collect all available events
                    let events: Vec<StreamEvent> = rx.iter().collect();
                    events.iter()
                        .filter_map(|e| {
                            if !e.text.is_empty() || e.is_done {
                                Some(stream_event_to_a2a_message(e))
                            } else {
                                None
                            }
                        })
                        .map(|msg| serde_json::to_value(&msg).ok())
                        .collect::<Option<Vec<_>>>()
                } else {
                    None
                }
            };

            let response_task = {
                let s = state_stream.lock().unwrap();
                s.build_task(&task_id).unwrap_or_else(|| req.task.clone())
            };

            Ok(serde_json::json!({
                "task": response_task,
                "streamEvents": events_json.unwrap_or_default()
            }))
        });

        // --- getTask handler ---
        let state_get = state.clone();
        let task_messages_get = task_messages.clone();

        self.server.register_handler("getTask", move |params| {
            let req: GetTaskRequest = serde_json::from_value(params)
                .map_err(|e| A2AError::invalid_params(format!("Invalid getTask params: {}", e)))?;

            let mut task = {
                let s = state_get.lock().unwrap();
                s.build_task(&req.id)
                    .ok_or_else(|| A2AError::task_not_found(&req.id))?
            };

            // Attach stored messages
            let msgs = task_messages_get.lock().unwrap();
            if let Some(stored) = msgs.get(&req.id) {
                task.messages = stored.clone();
            }

            Ok(serde_json::to_value(GetTaskResponse { task }).unwrap_or_default())
        });

        // --- cancelTask handler ---
        let state_cancel = state.clone();
        let adapter_cancel = self.adapter.clone();
        let active_streams_cancel = active_streams.clone();

        self.server.register_handler("cancelTask", move |params| {
            let req: CancelTaskRequest = serde_json::from_value(params)
                .map_err(|e| A2AError::invalid_params(format!("Invalid cancelTask params: {}", e)))?;

            adapter_cancel.cancel_task(&req.id)?;

            // Remove active stream if any
            {
                let mut streams = active_streams_cancel.lock().unwrap();
                streams.remove(&req.id);
            }

            let task = {
                let s = state_cancel.lock().unwrap();
                s.build_task(&req.id)
                    .ok_or_else(|| A2AError::task_not_found(&req.id))?
            };

            Ok(serde_json::to_value(CancelTaskResponse { task }).unwrap_or_default())
        });

        // --- listTasks handler ---
        let state_list = state;
        let task_messages_list = task_messages;

        self.server.register_handler("listTasks", move |_| {
            let tasks = {
                let s = state_list.lock().unwrap();
                s.build_all_tasks()
            };

            // Attach messages to each task
            let msgs = task_messages_list.lock().unwrap();
            let tasks_with_msgs: Vec<Task> = tasks.into_iter().map(|mut t| {
                if let Some(stored) = msgs.get(&t.id) {
                    t.messages = stored.clone();
                }
                t
            }).collect();

            Ok(serde_json::to_value(ListTasksResponse {
                tasks: tasks_with_msgs,
            }).unwrap_or_default())
        });
    }

    /// Handle an incoming HTTP request.
    ///
    /// For POST `/`: dispatch JSON-RPC request.
    /// For GET `/agent-card`: return the agent card.
    pub fn handle_http_request(&self, method: &str, path: &str, body: &str) -> String {
        if method == "GET" && path == "/agent-card" {
            return serde_json::to_string(self.server.agent_card()).unwrap_or_default();
        }

        // POST requests -> JSON-RPC dispatch
        self.server.handle_rpc_request(body)
    }
}

// ===========================================================================
// TCP/Unix Socket Server
// ===========================================================================

/// Configuration for the network listener.
#[derive(Debug, Clone)]
pub struct ListenerConfig {
    /// Address to bind to. Use "127.0.0.1" for TCP or a socket path for Unix.
    pub bind_address: String,
    /// Port to listen on (TCP only).
    pub port: u16,
    /// Whether to use Unix socket instead of TCP.
    pub use_unix_socket: bool,
    /// Socket directory for Unix sockets (default: ~/.agentszone/sock/).
    pub socket_dir: Option<String>,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 0, // 0 = auto-assign
            use_unix_socket: true,
            socket_dir: None,
        }
    }
}

impl ListenerConfig {
    /// Create a TCP listener config.
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self {
            bind_address: host.into(),
            port,
            use_unix_socket: false,
            socket_dir: None,
        }
    }

    /// Create a Unix socket listener config.
    pub fn unix_socket(socket_dir: impl Into<String>) -> Self {
        Self {
            bind_address: String::new(),
            port: 0,
            use_unix_socket: true,
            socket_dir: Some(socket_dir.into()),
        }
    }

    /// Get the default socket directory.
    pub fn default_socket_dir() -> String {
        dirs::home_dir()
            .map(|h| h.join(".agentszone").join("sock").to_string_lossy().to_string())
            .unwrap_or_else(|| "/tmp/agentszone".to_string())
    }

    /// Get the socket path for a given agent ID.
    pub fn socket_path(&self, agent_id: &str) -> String {
        let default_dir = Self::default_socket_dir();
        let dir = self.socket_dir.as_deref().unwrap_or(&default_dir);
        format!("{}/{}.sock", dir, agent_id)
    }
}

/// Start a simple HTTP listener for the adapter server.
///
/// This is a basic single-threaded HTTP server that handles one request at a time.
/// For production use, this would be replaced with an async server (hyper/axum).
pub fn start_tcp_listener(
    _server: &AdapterServer,
    config: &ListenerConfig,
) -> Result<TcpListener, A2AError> {
    let addr = format!("{}:{}", config.bind_address, config.port);
    let listener = TcpListener::bind(&addr)
        .map_err(|e| A2AError::internal_error(format!("Failed to bind to {}: {}", addr, e)))?;

    log::info!(
        "[AdapterServer] Listening on {}",
        listener.local_addr().unwrap()
    );

    Ok(listener)
}

/// Handle a single TCP connection.
pub fn handle_tcp_connection(
    server: &AdapterServer,
    stream: &mut std::net::TcpStream,
) -> Result<(), A2AError> {
    use std::io::{BufRead, BufReader};

    let mut reader = BufReader::new(stream.try_clone()
        .map_err(|e| A2AError::internal_error(format!("Failed to clone stream: {}", e)))?);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)
        .map_err(|e| A2AError::internal_error(format!("Failed to read request: {}", e)))?;

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return Err(A2AError::invalid_params("Malformed HTTP request"));
    }

    let method = parts[0];
    let path = parts[1];

    // Read headers
    let mut content_length = 0usize;
    loop {
        let mut header_line = String::new();
        reader.read_line(&mut header_line)
            .map_err(|e| A2AError::internal_error(format!("Failed to read header: {}", e)))?;
        if header_line.trim().is_empty() {
            break;
        }
        if let Some(cl) = header_line.strip_prefix("Content-Length:") {
            content_length = cl.trim().parse().unwrap_or(0);
        }
        if let Some(cl) = header_line.strip_prefix("content-length:") {
            content_length = cl.trim().parse().unwrap_or(0);
        }
    }

    // Read body
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)
            .map_err(|e| A2AError::internal_error(format!("Failed to read body: {}", e)))?;
    }

    let body_str = String::from_utf8_lossy(&body).to_string();
    let response_body = server.handle_http_request(method, path, &body_str);

    // Send HTTP response
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        response_body.len(),
        response_body
    );

    stream.write_all(response.as_bytes())
        .map_err(|e| A2AError::internal_error(format!("Failed to write response: {}", e)))?;
    stream.flush()
        .map_err(|e| A2AError::internal_error(format!("Failed to flush response: {}", e)))?;

    Ok(())
}

// ===========================================================================
// AgentCard generation
// ===========================================================================

/// Generate an AgentCard from adapter metadata and configuration.
pub fn generate_agent_card(
    adapter: &dyn CliA2AAdapter,
    agent_name: &str,
    description: Option<&str>,
    endpoint: Option<&str>,
) -> AgentCard {
    AgentCard {
        name: agent_name.to_string(),
        description: description.map(|s| s.to_string()),
        endpoint: endpoint.map(|s| s.to_string()),
        capabilities: adapter.capabilities(),
        supported_operations: vec![
            "sendMessage".to_string(),
            "streamMessage".to_string(),
            "getTask".to_string(),
            "cancelTask".to_string(),
            "listTasks".to_string(),
        ],
        auth: AuthInfo { schemes: vec![] },
        version: Some("1.0.0".to_string()),
    }
}

// ===========================================================================
// Socket File Cleanup — SocketGuard
// ===========================================================================

/// RAII guard that ensures a Unix socket file is cleaned up when dropped.
///
/// When the server shuts down (whether gracefully or due to a panic), the
/// `SocketGuard` will delete the socket file from disk. This prevents stale
/// socket files from accumulating in `~/.agentszone/sock/`.
///
/// # Usage
/// ```ignore
/// let socket_path = "/tmp/agentszone/my-agent.sock".to_string();
/// // Bind the socket listener here...
/// let _guard = SocketGuard::new(socket_path);
/// // When _guard goes out of scope, the file is removed.
/// ```
pub struct SocketGuard {
    /// The path to the socket file.
    socket_path: PathBuf,
    /// Whether the socket file was created by us (and should be cleaned up).
    owned: bool,
}

impl SocketGuard {
    /// Create a new socket guard for the given path.
    ///
    /// Does not create the file — only tracks the path for later cleanup.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: path.into(),
            owned: true,
        }
    }

    /// Create a guard that will NOT delete the file on drop.
    ///
    /// Useful when binding to a socket that is managed externally.
    pub fn borrowed(path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: path.into(),
            owned: false,
        }
    }

    /// Get the socket path.
    pub fn path(&self) -> &Path {
        &self.socket_path
    }

    /// Manually clean up the socket file now (instead of waiting for drop).
    ///
    /// Returns true if the file was removed or didn't exist.
    pub fn cleanup(&mut self) -> bool {
        if self.owned && self.socket_path.exists() {
            match std::fs::remove_file(&self.socket_path) {
                Ok(()) => {
                    log::info!(
                        "[SocketGuard] Cleaned up socket file: {}",
                        self.socket_path.display()
                    );
                    true
                }
                Err(e) => {
                    log::warn!(
                        "[SocketGuard] Failed to remove socket file {}: {}",
                        self.socket_path.display(),
                        e
                    );
                    false
                }
            }
        } else {
            true
        }
    }

    /// Check if the socket file currently exists on disk.
    pub fn exists(&self) -> bool {
        self.socket_path.exists()
    }
}

impl Drop for SocketGuard {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// ===========================================================================
// Connection Pool — manages reusable TCP connections for A2A communication
// ===========================================================================

/// An entry in the connection pool representing a single TCP connection.
#[derive(Debug)]
struct PoolEntry {
    /// The TCP stream.
    stream: std::net::TcpStream,
    /// When this entry was last used (epoch seconds).
    last_used: u64,
    /// Whether this entry is currently checked out.
    in_use: bool,
}

/// A simple connection pool for managing reusable TCP connections to A2A servers.
///
/// The pool:
/// - Maintains a bounded number of TCP connections per endpoint
/// - Reuses idle connections instead of creating new ones
/// - Evicts connections that have been idle too long
/// - Cleans up all connections when dropped
///
/// # Connection Lifecycle
/// ```text
/// acquire() → find idle or create new → check out → use → release() → check in
/// ```
pub struct ConnectionPool {
    /// Maximum connections per endpoint.
    max_per_endpoint: usize,
    /// Idle timeout in seconds; connections unused this long are evicted.
    idle_timeout_secs: u64,
    /// Map of endpoint -> pool entries.
    pools: Mutex<HashMap<String, Vec<PoolEntry>>>,
    /// Whether the pool is shutting down.
    shutting_down: AtomicBool,
}

/// Default pool constants.
const DEFAULT_MAX_PER_ENDPOINT: usize = 4;
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 300; // 5 minutes

impl ConnectionPool {
    /// Create a new connection pool with default settings.
    pub fn new() -> Self {
        Self {
            max_per_endpoint: DEFAULT_MAX_PER_ENDPOINT,
            idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
            pools: Mutex::new(HashMap::new()),
            shutting_down: AtomicBool::new(false),
        }
    }

    /// Create a pool with custom limits.
    pub fn with_limits(max_per_endpoint: usize, idle_timeout_secs: u64) -> Self {
        Self {
            max_per_endpoint,
            idle_timeout_secs,
            pools: Mutex::new(HashMap::new()),
            shutting_down: AtomicBool::new(false),
        }
    }

    /// Acquire a TCP connection to the given endpoint.
    ///
    /// Tries to reuse an existing idle connection. If none available and
    /// below the limit, creates a new one. Returns `None` if the pool is
    /// at capacity or shutting down.
    pub fn acquire(&self, endpoint: &str) -> Option<PooledConnection<'_>> {
        if self.shutting_down.load(Ordering::Relaxed) {
            return None;
        }

        let now = current_epoch_secs();
        let mut pools = self.pools.lock().unwrap();

        let entries = pools.entry(endpoint.to_string()).or_insert_with(Vec::new);

        // Try to find an idle connection
        for entry in entries.iter_mut() {
            if !entry.in_use {
                entry.in_use = true;
                entry.last_used = now;
                return Some(PooledConnection {
                    stream: Some(entry.stream.try_clone().ok()?),
                    endpoint: endpoint.to_string(),
                    pool: self,
                });
            }
        }

        // All entries in use — try to create a new one if below limit
        if entries.len() < self.max_per_endpoint {
            match std::net::TcpStream::connect(endpoint) {
                Ok(stream) => {
                    entries.push(PoolEntry {
                        stream: stream.try_clone().ok()?,
                        last_used: now,
                        in_use: true,
                    });
                    Some(PooledConnection {
                        stream: Some(stream),
                        endpoint: endpoint.to_string(),
                        pool: self,
                    })
                }
                Err(e) => {
                    log::warn!(
                        "[ConnectionPool] Failed to connect to {}: {}",
                        endpoint,
                        e
                    );
                    None
                }
            }
        } else {
            log::warn!(
                "[ConnectionPool] At capacity for endpoint {} ({}/{})",
                endpoint,
                entries.len(),
                self.max_per_endpoint
            );
            None
        }
    }

    /// Release a connection back to the pool.
    fn release(&self, endpoint: &str, stream: std::net::TcpStream) {
        let now = current_epoch_secs();
        let mut pools = self.pools.lock().unwrap();

        if let Some(entries) = pools.get_mut(endpoint) {
            // Find the entry matching this stream (by checking if it's in_use)
            // Since TcpStream doesn't implement PartialEq, we just mark an in_use slot
            for entry in entries.iter_mut() {
                if entry.in_use {
                    // Replace the stream in case it was cloned
                    entry.stream = stream;
                    entry.in_use = false;
                    entry.last_used = now;
                    return;
                }
            }
        }

        // If we can't find the entry, just drop the stream
        log::debug!(
            "[ConnectionPool] Released connection to {} but no matching pool entry",
            endpoint
        );
    }

    /// Evict idle connections that have exceeded the idle timeout.
    ///
    /// Returns the number of connections evicted.
    pub fn evict_idle(&self) -> usize {
        let now = current_epoch_secs();
        let mut pools = self.pools.lock().unwrap();
        let mut evicted = 0;

        for (_endpoint, entries) in pools.iter_mut() {
            let before = entries.len();
            entries.retain(|entry| {
                if entry.in_use {
                    return true; // Don't evict in-use connections
                }
                now.saturating_sub(entry.last_used) < self.idle_timeout_secs
            });
            evicted += before - entries.len();
        }

        if evicted > 0 {
            log::info!("[ConnectionPool] Evicted {} idle connections", evicted);
        }
        evicted
    }

    /// Get the total number of connections across all endpoints.
    pub fn total_connections(&self) -> usize {
        let pools = self.pools.lock().unwrap();
        pools.values().map(|entries| entries.len()).sum()
    }

    /// Get the number of active (in-use) connections.
    pub fn active_connections(&self) -> usize {
        let pools = self.pools.lock().unwrap();
        pools
            .values()
            .map(|entries| entries.iter().filter(|e| e.in_use).count())
            .sum()
    }

    /// Shut down the pool, closing all connections.
    pub fn shutdown(&self) {
        self.shutting_down.store(true, Ordering::Relaxed);
        let mut pools = self.pools.lock().unwrap();
        pools.clear();
        log::info!("[ConnectionPool] Pool shut down, all connections closed");
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ConnectionPool {
    fn drop(&mut self) {
        self.shutting_down.store(true, Ordering::Relaxed);
    }
}

/// A pooled connection that is automatically returned to the pool when dropped.
pub struct PooledConnection<'a> {
    stream: Option<std::net::TcpStream>,
    endpoint: String,
    pool: &'a ConnectionPool,
}

impl<'a> PooledConnection<'a> {
    /// Get a reference to the underlying TCP stream.
    pub fn stream(&self) -> Option<&std::net::TcpStream> {
        self.stream.as_ref()
    }

    /// Get a mutable reference to the underlying TCP stream.
    pub fn stream_mut(&mut self) -> Option<&mut std::net::TcpStream> {
        self.stream.as_mut()
    }
}

impl<'a> Drop for PooledConnection<'a> {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() {
            self.pool.release(&self.endpoint, stream);
        }
    }
}

/// Get current time as epoch seconds.
fn current_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::a2a::adapter::ClaudeCodeAdapter;

    fn test_agent_card() -> AgentCard {
        AgentCard {
            name: "TestAdapter".into(),
            description: Some("Test adapter".into()),
            endpoint: Some("http://localhost:9999/a2a".into()),
            capabilities: vec!["streaming".into()],
            supported_operations: vec!["sendMessage".into()],
            auth: AuthInfo { schemes: vec![] },
            version: Some("1.0.0".into()),
        }
    }

    #[test]
    fn test_adapter_server_new() {
        let adapter = Box::new(ClaudeCodeAdapter::new());
        let server = AdapterServer::new(adapter, test_agent_card());
        assert_eq!(server.agent_card().name, "TestAdapter");
    }

    #[test]
    fn test_adapter_server_handle_agent_card() {
        let adapter = Box::new(ClaudeCodeAdapter::new());
        let server = AdapterServer::new(adapter, test_agent_card());
        let response = server.handle_http_request("GET", "/agent-card", "");
        let card: AgentCard = serde_json::from_str(&response).unwrap();
        assert_eq!(card.name, "TestAdapter");
    }

    #[test]
    fn test_adapter_server_get_task_not_found() {
        let adapter = Box::new(ClaudeCodeAdapter::new());
        let server = AdapterServer::new(adapter, test_agent_card());
        server.register_adapter_handlers(AdapterConfig::default());

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getTask",
            "params": {"id": "nonexistent"},
            "id": 1
        }).to_string();

        let response = server.handle_http_request("POST", "/", &body);
        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert!(parsed.get("error").is_some());
        assert_eq!(parsed["error"]["code"], -32001);
    }

    #[test]
    fn test_adapter_server_list_tasks_empty() {
        let adapter = Box::new(ClaudeCodeAdapter::new());
        let server = AdapterServer::new(adapter, test_agent_card());
        server.register_adapter_handlers(AdapterConfig::default());

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "listTasks",
            "params": {},
            "id": 1
        }).to_string();

        let response = server.handle_http_request("POST", "/", &body);
        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
        let tasks = parsed["result"]["tasks"].as_array().unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_listener_config_default() {
        let config = ListenerConfig::default();
        assert!(config.use_unix_socket);
        assert_eq!(config.port, 0);
    }

    #[test]
    fn test_listener_config_tcp() {
        let config = ListenerConfig::tcp("0.0.0.0", 8080);
        assert!(!config.use_unix_socket);
        assert_eq!(config.bind_address, "0.0.0.0");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_listener_config_socket_path() {
        let config = ListenerConfig::unix_socket("/tmp/test");
        assert_eq!(config.socket_path("agent-1"), "/tmp/test/agent-1.sock");
    }

    #[test]
    fn test_generate_agent_card() {
        let adapter = ClaudeCodeAdapter::new();
        let card = generate_agent_card(
            &adapter,
            "MyAgent",
            Some("A test agent"),
            Some("http://localhost:8080/a2a"),
        );
        assert_eq!(card.name, "MyAgent");
        assert_eq!(card.description, Some("A test agent".to_string()));
        assert_eq!(card.endpoint, Some("http://localhost:8080/a2a".to_string()));
        assert!(card.capabilities.contains(&"streaming".to_string()));
        assert!(card.supported_operations.contains(&"sendMessage".to_string()));
        assert!(card.supported_operations.contains(&"cancelTask".to_string()));
    }

    // --- SocketGuard tests ---

    #[test]
    fn test_socket_guard_cleanup_on_drop() {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");

        // Create a fake socket file
        std::fs::write(&socket_path, "").unwrap();
        assert!(socket_path.exists());

        // Create guard and let it drop
        {
            let _guard = SocketGuard::new(&socket_path);
        }

        // File should be cleaned up
        assert!(!socket_path.exists());
    }

    #[test]
    fn test_socket_guard_no_cleanup_for_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("nonexistent.sock");

        assert!(!socket_path.exists());

        // Should not panic
        let _guard = SocketGuard::new(&socket_path);
    }

    #[test]
    fn test_socket_guard_borrowed_does_not_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("borrowed.sock");

        std::fs::write(&socket_path, "").unwrap();
        assert!(socket_path.exists());

        {
            let _guard = SocketGuard::borrowed(&socket_path);
        }

        // Borrowed guard should NOT clean up
        assert!(socket_path.exists());
    }

    #[test]
    fn test_socket_guard_manual_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("manual.sock");

        std::fs::write(&socket_path, "").unwrap();
        assert!(socket_path.exists());

        let mut guard = SocketGuard::new(&socket_path);
        assert!(guard.cleanup());
        assert!(!socket_path.exists());

        // Second cleanup should be a no-op
        assert!(guard.cleanup());
    }

    #[test]
    fn test_socket_guard_path() {
        let guard = SocketGuard::new("/tmp/test.sock");
        assert_eq!(guard.path(), std::path::Path::new("/tmp/test.sock"));
    }

    // --- ConnectionPool tests ---

    #[test]
    fn test_connection_pool_new() {
        let pool = ConnectionPool::new();
        assert_eq!(pool.total_connections(), 0);
        assert_eq!(pool.active_connections(), 0);
    }

    #[test]
    fn test_connection_pool_default() {
        let pool = ConnectionPool::default();
        assert_eq!(pool.total_connections(), 0);
    }

    #[test]
    fn test_connection_pool_with_limits() {
        let pool = ConnectionPool::with_limits(2, 60);
        assert_eq!(pool.total_connections(), 0);
    }

    #[test]
    fn test_connection_pool_shutdown() {
        let pool = ConnectionPool::new();
        pool.shutdown();
        assert_eq!(pool.total_connections(), 0);
    }

    #[test]
    fn test_connection_pool_acquire_nonexistent() {
        let pool = ConnectionPool::new();
        // Trying to connect to a non-existent endpoint should return None
        let result = pool.acquire("127.0.0.1:1");
        assert!(result.is_none());
    }

    #[test]
    fn test_connection_pool_acquire_after_shutdown() {
        let pool = ConnectionPool::new();
        pool.shutdown();
        // Should not acquire after shutdown
        let result = pool.acquire("127.0.0.1:8080");
        assert!(result.is_none());
    }

    #[test]
    fn test_connection_pool_evict_idle_empty() {
        let pool = ConnectionPool::new();
        let evicted = pool.evict_idle();
        assert_eq!(evicted, 0);
    }

    #[test]
    fn test_connection_pool_acquire_and_release_via_tcp() {
        // Start a local TCP listener for the test
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let endpoint = format!("127.0.0.1:{}", addr.port());

        let pool = ConnectionPool::new();

        // Accept the connection in the background
        let accept_handle = std::thread::spawn(move || {
            let _ = listener.accept();
        });

        let conn = pool.acquire(&endpoint);
        assert!(conn.is_some());
        assert_eq!(pool.active_connections(), 1);

        // Drop the connection (returns it to pool)
        drop(conn);
        assert_eq!(pool.active_connections(), 0);

        accept_handle.join().unwrap();
    }

    #[test]
    fn test_connection_pool_respects_max_connections() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let endpoint = format!("127.0.0.1:{}", addr.port());

        let pool = ConnectionPool::with_limits(1, 60);

        // Accept connections in the background
        let accept_handle = std::thread::spawn(move || {
            // Accept as many as come, but we only need 2 attempts
            let _ = listener.accept();
            // Second one may time out, that's fine
        });

        let conn1 = pool.acquire(&endpoint);
        assert!(conn1.is_some());

        // Pool is at capacity (1/1), second acquire should fail
        // (all entries are in_use)
        let conn2 = pool.acquire(&endpoint);
        assert!(conn2.is_none());

        drop(conn1);
        drop(conn2);

        accept_handle.join().unwrap();
    }
}
