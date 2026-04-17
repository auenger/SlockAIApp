//! BridgeServer — combines AdapterServer with AgentManager for remote workspace gateway.

use std::sync::{Arc, Mutex};

use crate::runtime::a2a::adapter::cli_adapter::{AdapterConfig, CliA2AAdapter};
use crate::runtime::a2a::adapter::handler::{run_adapter_server_loop, AdapterServer, ListenerConfig};
use crate::runtime::a2a::adapter::ClaudeCodeAdapter;
use crate::runtime::a2a::types::AgentCard;
use crate::workspace::manager::AgentManager;

use super::config::BridgeConfig;

/// Bridge server — the core of the remote workspace gateway.
///
/// Combines:
/// - An `AdapterServer` for standard A2A protocol handling
/// - An `AgentManager` for workspace/agent management
/// - Extended bridge.* protocol handlers
pub struct BridgeServer {
    /// Standard A2A adapter server.
    adapter_server: Arc<AdapterServer>,
    /// Workspace agent manager.
    agent_manager: Arc<Mutex<AgentManager>>,
    /// Bridge configuration.
    config: BridgeConfig,
}

impl BridgeServer {
    /// Create and initialize a new BridgeServer.
    ///
    /// 1. Creates AgentManager and initializes workspace
    /// 2. Loads agents from disk
    /// 3. Creates AdapterServer with ClaudeCodeAdapter
    /// 4. Registers standard A2A handlers
    /// 5. Registers bridge.* extension handlers
    pub fn new(config: BridgeConfig) -> Result<Self, String> {
        // Initialize workspace
        let mut agent_manager = AgentManager::new(&config.workspace_root);
        agent_manager
            .initialize_workspace()
            .map_err(|e| format!("Workspace initialization failed: {}", e))?;

        agent_manager
            .load()
            .map_err(|e| format!("Failed to load agents: {}", e))?;

        let agent_count = agent_manager.list_agents().len();
        log::info!("[BridgeServer] Loaded {} agents from workspace", agent_count);

        let agent_manager = Arc::new(Mutex::new(agent_manager));

        // Create AdapterServer with ClaudeCodeAdapter
        let claude_adapter = ClaudeCodeAdapter::new();
        let capabilities = claude_adapter.capabilities();
        let adapter: Box<dyn CliA2AAdapter> = Box::new(claude_adapter);

        let agent_card = AgentCard {
            name: config.name.clone(),
            description: Some("Remote Workspace Bridge".to_string()),
            endpoint: None,
            capabilities,
            supported_operations: vec![
                "sendMessage".to_string(),
                "streamMessage".to_string(),
                "getTask".to_string(),
                "cancelTask".to_string(),
                "listTasks".to_string(),
                "bridge.getWorkspaceInfo".to_string(),
                "bridge.getAgents".to_string(),
                "bridge.listFiles".to_string(),
                "bridge.readFile".to_string(),
            ],
            auth: crate::runtime::a2a::types::AuthInfo { schemes: vec![] },
            version: Some("1.0.0".to_string()),
        };

        let adapter_server = Arc::new(AdapterServer::new(adapter, agent_card));

        // Register standard A2A handlers
        adapter_server.register_adapter_handlers(AdapterConfig::default());

        // Register bridge.* extension handlers
        super::handlers::register_bridge_handlers(&adapter_server, &agent_manager);

        Ok(Self {
            adapter_server,
            agent_manager,
            config,
        })
    }

    /// Run the bridge server TCP accept loop.
    ///
    /// Blocks until shutdown is signaled.
    /// Returns Ok(()) on graceful shutdown.
    pub fn run(&self) -> Result<(), String> {
        let config = ListenerConfig::tcp(&self.config.bind, self.config.port);

        let (_handle, shutdown, done_rx) = run_adapter_server_loop(self.adapter_server.clone(), config)
            .map_err(|e| format!("Failed to start server: {}", e))?;

        // Print startup information
        let agent_count = {
            let mgr = self.agent_manager.lock().unwrap();
            mgr.list_agents().len()
        };

        println!("[az-bridge] Listening on {}:{}", self.config.bind, self.config.port);
        println!("[az-bridge] Workspace: {}", self.config.workspace_root.display());
        println!("[az-bridge] Agents: {}", agent_count);
        println!("[az-bridge] Name: {}", self.config.name);

        let local_ips = get_local_ip_addresses();
        if !local_ips.is_empty() {
            println!("[az-bridge] Local IPs: {}", local_ips.join(", "));
        }

        println!("[az-bridge] Press Ctrl+C to stop");

        // Set up Ctrl+C handler
        let shutdown_signal = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shutdown_signal_clone = shutdown_signal.clone();
        let shutdown_server = shutdown.clone();

        ctrlc_handler(move || {
            println!("\n[az-bridge] Shutting down...");
            shutdown_server.store(true, std::sync::atomic::Ordering::Relaxed);
            shutdown_signal_clone.store(true, std::sync::atomic::Ordering::Relaxed);
        });

        // Wait for shutdown signal (blocks until Ctrl+C)
        let _ = done_rx.recv();
        println!("[az-bridge] Server stopped");
        Ok(())
    }
}

// ===========================================================================
// Helpers (copied from cli.rs pattern)
// ===========================================================================

/// Get local IP addresses for display.
fn get_local_ip_addresses() -> Vec<String> {
    let mut ips = Vec::new();

    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        let targets = ["8.8.8.8:80", "1.1.1.1:80", "208.67.222.222:80"];
        let mut seen = std::collections::HashSet::new();

        for target in &targets {
            if socket.connect(target).is_ok() {
                if let Ok(local_addr) = socket.local_addr() {
                    let ip = local_addr.ip().to_string();
                    if ip != "127.0.0.1" && !ip.starts_with("0.") && seen.insert(ip.clone()) {
                        ips.push(ip);
                    }
                }
            }
        }
    }

    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }

    ips
}

/// Set up a Ctrl+C handler using libc signals (Unix only).
fn ctrlc_handler<F: Fn() + Send + 'static>(handler: F) {
    #[cfg(unix)]
    {
        use std::sync::Once;
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            unsafe {
                libc_signal_init();
            }
        });

        std::thread::spawn(move || {
            unsafe {
                wait_for_sigint();
            }
            handler();
        });
    }

    #[cfg(windows)]
    {
        use std::sync::Once;
        static INIT: Once = Once::new();
        static mut HANDLER: Option<Box<dyn Fn() + Send + 'static>> = None;

        INIT.call_once(|| {
            unsafe {
                HANDLER = Some(Box::new(handler));
            }

            // Set Windows console Ctrl+C handler
            extern "system" fn console_ctrl_handler(ctrl_type: u32) -> i32 {
                if ctrl_type == 0 {
                    // CTRL_C_EVENT
                    unsafe {
                        if let Some(ref h) = HANDLER {
                            h();
                        }
                    }
                    1
                } else {
                    0
                }
            }

            unsafe {
                windows_set_console_ctrl_handler(console_ctrl_handler);
            }
        });
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = handler;
    }
}

#[cfg(unix)]
mod unix_signal {
    use std::sync::atomic::{AtomicBool, Ordering};

    static SIGINT_RECEIVED: AtomicBool = AtomicBool::new(false);

    extern "C" fn sigint_handler(_sig: i32) {
        SIGINT_RECEIVED.store(true, Ordering::Relaxed);
    }

    pub unsafe fn libc_signal_init() {
        libc::signal(libc::SIGINT, sigint_handler as *const () as usize);
    }

    pub unsafe fn wait_for_sigint() {
        while !SIGINT_RECEIVED.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

#[cfg(unix)]
use unix_signal::{libc_signal_init, wait_for_sigint};

#[cfg(windows)]
fn windows_set_console_ctrl_handler(
    handler: extern "system" fn(u32) -> i32,
) {
    use std::os::raw::c_int;
    type HandlerRoutine = extern "system" fn(u32) -> i32;
    extern "system" {
        fn SetConsoleCtrlHandler(
            handler_routine: Option<HandlerRoutine>,
            add: c_int,
        ) -> c_int;
    }
    unsafe {
        SetConsoleCtrlHandler(Some(handler), 1);
    }
}
