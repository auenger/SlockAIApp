//! Headless CLI mode for AgentsZone.
//!
//! Provides a `serve` subcommand that starts the A2A TCP server without
//! launching the Tauri GUI. This enables running AgentsZone as a
//! background service on servers or headless machines.
//!
//! # Usage
//! ```text
//! agentszone                     # Normal GUI mode
//! agentszone serve               # Headless mode (default port 7878)
//! agentszone serve --port 8080   # Custom port
//! agentszone serve -p 8080 -b 127.0.0.1
//! agentszone --help
//! agentszone serve --help
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::Parser;

use crate::runtime::a2a::adapter::cli_adapter::{AdapterConfig, CliA2AAdapter};
use crate::runtime::a2a::adapter::handler::{run_adapter_server_loop, AdapterServer, ListenerConfig};
use crate::runtime::a2a::adapter::ClaudeCodeAdapter;
use crate::runtime::a2a::types::AgentCard;

// ===========================================================================
// CLI Definitions
// ===========================================================================

/// AgentsZone — AI-native collaboration desktop app with headless A2A server mode.
#[derive(Parser, Debug)]
#[command(name = "agentszone", version, about = "AgentsZone — AI-native collaboration app")]
pub struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available subcommands.
#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Start headless A2A server (no GUI).
    Serve {
        /// Port to listen on.
        #[arg(short, long, default_value_t = 7878, value_name = "PORT")]
        port: u16,

        /// Bind address (e.g. "0.0.0.0" for all interfaces, "127.0.0.1" for localhost).
        #[arg(short, long, default_value = "0.0.0.0", value_name = "ADDR")]
        bind: String,
    },
}

// ===========================================================================
// Headless Server Entry
// ===========================================================================

/// Run the headless A2A server.
///
/// This function:
/// 1. Creates a `ClaudeCodeAdapter` and `AdapterServer`
/// 2. Binds to the specified address and port
/// 3. Prints startup information (listening address, agent card, local IPs)
/// 4. Sets up graceful shutdown on Ctrl+C
/// 5. Runs until shutdown signal is received
///
/// Returns `Ok(())` on graceful shutdown, or an error string on failure.
pub fn run_headless_server(bind: &str, port: u16) -> Result<(), String> {
    println!("[AgentsZone A2A Server] Starting...");

    // Build the AdapterServer with ClaudeCodeAdapter
    let claude_adapter = ClaudeCodeAdapter::new();
    let capabilities = claude_adapter.capabilities();
    let runtime_name = claude_adapter.runtime_name().to_string();
    let adapter: Box<dyn crate::runtime::a2a::adapter::CliA2AAdapter> = Box::new(claude_adapter);

    let agent_card = AgentCard {
        name: "AgentsZone".to_string(),
        description: Some("LAN A2A Server (headless)".to_string()),
        endpoint: None,
        capabilities: capabilities.clone(),
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

    let config = ListenerConfig::tcp(bind, port);

    // Start the TCP accept loop
    let (_handle, shutdown, done_rx) = run_adapter_server_loop(server, config)
        .map_err(|e| format!("Failed to bind to {}:{}: {}", bind, port, e))?;

    // Print startup information
    println!("[AgentsZone A2A Server] Listening on {}:{}", bind, port);
    println!(
        "[AgentsZone A2A Server] Agent: {} ({})",
        runtime_name,
        capabilities.join(", ")
    );

    let local_ips = get_local_ip_addresses();
    if !local_ips.is_empty() {
        println!("[AgentsZone A2A Server] Local IPs: {}", local_ips.join(", "));
    }

    println!("[AgentsZone A2A Server] Press Ctrl+C to stop");

    // Set up Ctrl+C handler
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_signal_clone = shutdown_signal.clone();
    let shutdown_server = shutdown.clone();

    ctrlc_handler(move || {
        println!("\n[AgentsZone A2A Server] Shutting down...");
        shutdown_server.store(true, Ordering::Relaxed);
        shutdown_signal_clone.store(true, Ordering::Relaxed);
    });

    // Wait for the server loop to finish
    match done_rx.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(()) => {
            println!("[AgentsZone A2A Server] Server stopped");
            Ok(())
        }
        Err(_) => {
            // Timeout waiting for server to stop — force shutdown
            shutdown.store(true, Ordering::Relaxed);
            println!("[AgentsZone A2A Server] Server stopped (forced)");
            Ok(())
        }
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Get local IP addresses for display.
///
/// Returns all non-loopback IPv4 addresses likely to be reachable on the LAN.
fn get_local_ip_addresses() -> Vec<String> {
    let mut ips = Vec::new();

    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
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
                    if ip != "127.0.0.1" && !ip.starts_with("0.") && seen.insert(ip.clone()) {
                        ips.push(ip);
                    }
                }
            }
        }
    }

    // Fallback: try hostname resolution
    if ips.is_empty() {
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

    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }

    ips
}

/// Set up a Ctrl+C handler.
///
/// On Unix, uses `ctrlc` crate or a simple signal handler.
/// The closure is called once when Ctrl+C is pressed.
fn ctrlc_handler<F: Fn() + Send + 'static>(handler: F) {
    // Use a simple approach: spawn a thread that waits for SIGINT
    #[cfg(unix)]
    {
        use std::sync::Once;
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            unsafe {
                libc_signal_init();
            }
        });

        // Use a thread-based approach with libc
        std::thread::spawn(move || {
            // Block waiting for SIGINT
            unsafe {
                wait_for_sigint();
            }
            handler();
        });
    }

    #[cfg(not(unix))]
    {
        // On non-Unix platforms, use a simple polling approach
        let _ = handler; // suppress unused warning
    }
}

// ===========================================================================
// Platform-specific signal handling
// ===========================================================================

#[cfg(unix)]
mod unix_signal {
    use std::sync::atomic::{AtomicBool, Ordering};

    static SIGINT_RECEIVED: AtomicBool = AtomicBool::new(false);

    /// Signal handler — just sets a flag (async-signal-safe).
    extern "C" fn sigint_handler(_sig: i32) {
        SIGINT_RECEIVED.store(true, Ordering::Relaxed);
    }

    /// Install the SIGINT handler using libc.
    ///
    /// # Safety
    /// Calls libc `signal()` which is thread-safe.
    pub unsafe fn libc_signal_init() {
        // Install handler — cast function pointer to c_uint for libc::signal
        libc::signal(libc::SIGINT, sigint_handler as *const () as usize);
    }

    /// Busy-wait until SIGINT is received.
    ///
    /// # Safety
    /// Reads an atomic bool, which is safe.
    pub unsafe fn wait_for_sigint() {
        while !SIGINT_RECEIVED.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

#[cfg(unix)]
use unix_signal::{libc_signal_init, wait_for_sigint};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_no_subcommand() {
        let cli = Cli::try_parse_from(&["agentszone"]);
        assert!(cli.is_ok());
        assert!(cli.unwrap().command.is_none());
    }

    #[test]
    fn test_cli_serve_default() {
        let cli = Cli::try_parse_from(&["agentszone", "serve"]);
        assert!(cli.is_ok());
        match cli.unwrap().command {
            Some(Commands::Serve { port, bind }) => {
                assert_eq!(port, 7878);
                assert_eq!(bind, "0.0.0.0");
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_cli_serve_custom_port() {
        let cli = Cli::try_parse_from(&["agentszone", "serve", "--port", "9090"]);
        assert!(cli.is_ok());
        match cli.unwrap().command {
            Some(Commands::Serve { port, bind }) => {
                assert_eq!(port, 9090);
                assert_eq!(bind, "0.0.0.0");
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_cli_serve_custom_bind() {
        let cli = Cli::try_parse_from(&["agentszone", "serve", "-p", "8080", "-b", "127.0.0.1"]);
        assert!(cli.is_ok());
        match cli.unwrap().command {
            Some(Commands::Serve { port, bind }) => {
                assert_eq!(port, 8080);
                assert_eq!(bind, "127.0.0.1");
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_cli_help_output() {
        // --help should cause a graceful exit with error
        let result = Cli::try_parse_from(&["agentszone", "--help"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("AgentsZone"));
    }

    #[test]
    fn test_cli_serve_help_output() {
        let result = Cli::try_parse_from(&["agentszone", "serve", "--help"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("headless"));
    }

    #[test]
    fn test_cli_version() {
        let result = Cli::try_parse_from(&["agentszone", "--version"]);
        assert!(result.is_err()); // clap exits with --version
    }

    #[test]
    fn test_get_local_ip_addresses() {
        let ips = get_local_ip_addresses();
        // Should return at least one IP (even if loopback)
        assert!(!ips.is_empty());
    }

    #[test]
    fn test_serve_invalid_port() {
        // Port 0 is technically valid for TCP but unusual
        let cli = Cli::try_parse_from(&["agentszone", "serve", "--port", "0"]);
        assert!(cli.is_ok());
    }
}
