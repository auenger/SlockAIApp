// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

fn main() {
    let cli = agentszone_lib::cli::Cli::parse();

    match &cli.command {
        Some(agentszone_lib::cli::Commands::Serve { port, bind }) => {
            // Headless mode: run A2A server without GUI
            if let Err(e) = agentszone_lib::cli::run_headless_server(bind, *port) {
                eprintln!("[AgentsZone A2A Server] Error: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            // Normal GUI mode
            agentszone_lib::run();
        }
    }
}
