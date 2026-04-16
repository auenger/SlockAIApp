//! Tauri IPC commands for managing the LAN A2A server lifecycle.
//!
//! Provides start/stop/status commands for the embedded A2A TCP server,
//! as well as a helper to list local IP addresses for sharing with
//! other devices on the LAN.

use serde::Serialize;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::runtime::a2a::adapter::cli_adapter::AdapterConfig;
use crate::runtime::a2a::adapter::handler::{
    run_adapter_server_loop, AdapterServer, ListenerConfig,
};
use crate::runtime::a2a::adapter::ClaudeCodeAdapter;
use crate::runtime::a2a::types::AgentCard;

// ===========================================================================
// Types
// ===========================================================================

/// Running state of the A2A TCP server.
struct RunningServer {
    /// Flag to signal the server loop to shut down.
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    /// Receiver that yields () once the accept loop thread has exited.
    done_rx: std::sync::mpsc::Receiver<()>,
    /// Port the server is listening on.
    port: u16,
}

/// Managed state for the A2A server lifecycle.
pub struct A2AServerState {
    /// The running server, if any.
    running: Option<RunningServer>,
}

impl A2AServerState {
    pub fn new() -> Self {
        Self { running: None }
    }
}

/// Status of the A2A LAN server.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LanServerStatus {
    /// Server is running and accepting connections.
    Running,
    /// Server is stopped.
    Stopped,
    /// Server encountered an error.
    Error(String),
}

/// Information about the running A2A LAN server.
#[derive(Debug, Clone, Serialize)]
pub struct LanServerInfo {
    /// Status of the server.
    pub status: LanServerStatus,
    /// Port the server is bound to.
    pub port: u16,
    /// Local IP addresses that can reach this server.
    pub local_ips: Vec<String>,
    /// URL for the agent card endpoint.
    pub agent_card_url: Option<String>,
}

impl Default for A2AServerState {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Commands
// ===========================================================================

/// Start the A2A LAN server on the given port.
///
/// Creates an `AdapterServer` backed by a `ClaudeCodeAdapter`, starts
/// the TCP accept loop on a background thread, and stores the handle
/// in managed state.
#[tauri::command]
pub fn start_a2a_server(
    state: tauri::State<'_, AppState>,
    port: u16,
) -> Result<LanServerInfo, String> {
    // Check if already running
    {
        let a2a_state = state.a2a_server.lock().map_err(|e| format!("lock error: {}", e))?;
        if a2a_state.running.is_some() {
            let running = a2a_state.running.as_ref().unwrap();
            return Ok(LanServerInfo {
                status: LanServerStatus::Running,
                port: running.port,
                local_ips: get_local_ip_addresses_inner(),
                agent_card_url: Some(format!("http://{{}}:{}/agent-card", running.port)),
            });
        }
    }

    // Build the AdapterServer with ClaudeCodeAdapter
    let adapter = Box::new(ClaudeCodeAdapter::new());
    let agent_card = AgentCard {
        name: "AgentsZone".to_string(),
        description: Some("LAN A2A Server".to_string()),
        endpoint: None,
        capabilities: vec!["streaming".to_string()],
        supported_operations: vec![
            "sendMessage".to_string(),
            "streamMessage".to_string(),
            "getTask".to_string(),
            "cancelTask".to_string(),
            "listTasks".to_string(),
        ],
        auth: crate::runtime::a2a::types::AuthInfo { schemes: vec![] },
        version: Some("1.0.0".to_string()),
    };

    let server = Arc::new(AdapterServer::new(adapter, agent_card));
    server.register_adapter_handlers(AdapterConfig::default());

    let config = ListenerConfig::tcp("0.0.0.0", port);

    let (_handle, shutdown, done_rx) = run_adapter_server_loop(server, config)
        .map_err(|e| format!("Failed to start A2A server: {}", e))?;

    // Determine actual bound port
    let actual_port = port;

    log::info!(
        "[a2a_server] LAN A2A server started on 0.0.0.0:{}",
        actual_port
    );

    // Store in state
    {
        let mut a2a_state = state.a2a_server.lock().map_err(|e| format!("lock error: {}", e))?;
        a2a_state.running = Some(RunningServer {
            shutdown,
            done_rx,
            port: actual_port,
        });
    }

    Ok(LanServerInfo {
        status: LanServerStatus::Running,
        port: actual_port,
        local_ips: get_local_ip_addresses_inner(),
        agent_card_url: Some(format!("http://{{}}:{}/agent-card", actual_port)),
    })
}

/// Stop the A2A LAN server.
#[tauri::command]
pub fn stop_a2a_server(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut a2a_state = state.a2a_server.lock().map_err(|e| format!("lock error: {}", e))?;

    if let Some(running) = a2a_state.running.take() {
        running.shutdown.store(true, Ordering::Relaxed);
        // Wait for the accept loop to finish (with timeout)
        match running.done_rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(()) => log::info!("[a2a_server] Server stopped gracefully"),
            Err(_) => log::warn!("[a2a_server] Server stop timed out, forcing"),
        }
        log::info!("[a2a_server] LAN A2A server stopped on port {}", running.port);
    }

    Ok(())
}

/// Get the current status of the A2A LAN server.
#[tauri::command]
pub fn get_a2a_server_status(
    state: tauri::State<'_, AppState>,
) -> Result<LanServerInfo, String> {
    let a2a_state = state.a2a_server.lock().map_err(|e| format!("lock error: {}", e))?;

    match &a2a_state.running {
        Some(running) => Ok(LanServerInfo {
            status: LanServerStatus::Running,
            port: running.port,
            local_ips: get_local_ip_addresses_inner(),
            agent_card_url: Some(format!("http://{{}}:{}/agent-card", running.port)),
        }),
        None => Ok(LanServerInfo {
            status: LanServerStatus::Stopped,
            port: 0,
            local_ips: vec![],
            agent_card_url: None,
        }),
    }
}

/// Get the local IP addresses of this machine.
///
/// Returns all non-loopback IPv4 addresses that are likely to be
/// reachable on the LAN.
#[tauri::command]
pub fn get_local_ip_addresses() -> Result<Vec<String>, String> {
    Ok(get_local_ip_addresses_inner())
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Inner implementation for getting local IP addresses.
fn get_local_ip_addresses_inner() -> Vec<String> {
    let mut ips = Vec::new();

    // Use a UDP socket trick to determine local IPs without actually sending data
    // Connect a UDP socket to a public IP (doesn't actually send data)
    // then read the local address
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        // Try connecting to a few well-known public IPs to enumerate interfaces
        let targets = [
            "8.8.8.8:80",
            "1.1.1.1:80",
            "208.67.222.222:80",
        ];

        let mut seen = std::collections::HashSet::new();

        for target in &targets {
            if socket.connect(target).is_ok() {
                if let Ok(local_addr) = socket.local_addr() {
                    let ip = local_addr.ip().to_string();
                    // Skip loopback
                    if ip != "127.0.0.1" && !ip.starts_with("0.") && seen.insert(ip.clone()) {
                        ips.push(ip);
                    }
                }
            }
        }
    }

    // If no IPs found via UDP trick, fall back to listing interfaces
    if ips.is_empty() {
        // Try to get IP from hostname resolution
        if let Ok(hostname) = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
        {
            if let Ok(addrs) = std::net::ToSocketAddrs::to_socket_addrs(
                &format!("{}:0", hostname)
            ) {
                for addr in addrs {
                    let ip = addr.ip().to_string();
                    if ip != "127.0.0.1" && !ip.contains(':') {
                        ips.push(ip);
                    }
                }
            }
        }
    }

    // If still empty, return loopback as fallback
    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }

    ips
}

// Import AppState from the parent module
use super::AppState;
