//! az-bridge — Remote Workspace Gateway binary.
//!
//! Standalone A2A bridge that exposes a local workspace as a remote endpoint.
//! No Tauri/GUI dependencies — compiled with `--no-default-features`.
//!
//! # Usage
//! ```text
//! az-bridge                                    # default: 0.0.0.0:7878
//! az-bridge --port 9090                        # custom port
//! az-bridge -b 127.0.0.1 -p 8080              # custom bind + port
//! az-bridge --workspace ~/my-workspace         # custom workspace root
//! az-bridge --config /path/to/bridge.toml      # custom config file
//! ```

use clap::Parser;

/// Remote Workspace Gateway — expose your workspace via A2A protocol.
#[derive(Parser, Debug)]
#[command(name = "az-bridge", version, about = "AgentsZone Remote Workspace Gateway")]
struct Args {
    /// Port to listen on.
    #[arg(short, long, value_name = "PORT")]
    port: Option<u16>,

    /// Bind address (e.g. "0.0.0.0" for all, "127.0.0.1" for localhost).
    #[arg(short, long, value_name = "ADDR")]
    bind: Option<String>,

    /// Path to TOML config file.
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Workspace root directory.
    #[arg(short, long, value_name = "DIR")]
    workspace: Option<String>,
}

fn main() {
    let args = Args::parse();

    println!("[az-bridge] AgentsZone Remote Workspace Gateway");
    println!("[az-bridge] Initializing...");

    // Resolve configuration: CLI > TOML > defaults
    let config = match agentszone_lib::bridge::BridgeConfig::resolve(
        args.port,
        args.bind.as_deref(),
        args.config.as_deref(),
        args.workspace.as_deref(),
    ) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[az-bridge] ERROR: Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    // Create and run the bridge server
    let server = match agentszone_lib::bridge::BridgeServer::new(config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[az-bridge] ERROR: Failed to initialize: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = server.run() {
        eprintln!("[az-bridge] ERROR: {}", e);
        std::process::exit(1);
    }
}
