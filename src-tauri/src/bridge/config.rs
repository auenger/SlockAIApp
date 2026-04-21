//! Bridge configuration — CLI args + TOML file + defaults.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Bridge server configuration.
///
/// Configuration priority: CLI args > TOML file > defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// Root workspace directory (default: ~/.agentszone).
    pub workspace_root: PathBuf,
    /// Bind address (default: "0.0.0.0").
    pub bind: String,
    /// Listen port (default: 7878).
    pub port: u16,
    /// Display name for this bridge (default: hostname).
    pub name: String,
    /// Runtime binary overrides (e.g. claude_binary = "claude.cmd").
    #[serde(default)]
    pub runtime: RuntimeConfig,
}

/// Runtime binary name overrides for the bridge host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Binary name for Claude Code CLI (default: "claude").
    #[serde(default = "RuntimeConfig::default_claude_binary")]
    pub claude_binary: String,
    /// Binary name for Codex CLI (default: "codex").
    #[serde(default = "RuntimeConfig::default_codex_binary")]
    pub codex_binary: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            claude_binary: Self::default_claude_binary(),
            codex_binary: Self::default_codex_binary(),
        }
    }
}

impl RuntimeConfig {
    fn default_claude_binary() -> String { "claude".to_string() }
    fn default_codex_binary() -> String { "codex".to_string() }
}

impl Default for BridgeConfig {
    fn default() -> Self {
        let workspace_root = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".agentszone");

        let name = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "az-bridge".to_string());

        Self {
            workspace_root,
            bind: "0.0.0.0".to_string(),
            port: 7878,
            name,
            runtime: RuntimeConfig::default(),
        }
    }
}

/// TOML file structure for bridge configuration.
#[derive(Debug, Deserialize)]
struct BridgeToml {
    bridge: Option<BridgeTomlSection>,
    runtime: Option<RuntimeTomlSection>,
}

#[derive(Debug, Deserialize)]
struct BridgeTomlSection {
    workspace_root: Option<String>,
    bind: Option<String>,
    port: Option<u16>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RuntimeTomlSection {
    claude_binary: Option<String>,
    codex_binary: Option<String>,
}

impl BridgeConfig {
    /// Load configuration with priority: CLI > TOML > defaults.
    ///
    /// - `cli_port`: port from CLI `--port`
    /// - `cli_bind`: bind address from CLI `--bind`
    /// - `cli_config`: path to TOML config file (default: ~/.agentszone/bridge.toml)
    /// - `cli_workspace`: workspace root from CLI `--workspace`
    pub fn resolve(
        cli_port: Option<u16>,
        cli_bind: Option<&str>,
        cli_config: Option<&str>,
        cli_workspace: Option<&str>,
    ) -> Result<Self, String> {
        let mut config = Self::default();

        // Load TOML config if available
        let config_path = cli_config
            .map(PathBuf::from)
            .unwrap_or_else(|| config.workspace_root.join("bridge.toml"));

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config {}: {}", config_path.display(), e))?;
            let toml: BridgeToml = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse config {}: {}", config_path.display(), e))?;

            if let Some(section) = toml.bridge {
                if let Some(root) = section.workspace_root {
                    config.workspace_root = PathBuf::from(root);
                }
                if let Some(bind) = section.bind {
                    config.bind = bind;
                }
                if let Some(port) = section.port {
                    config.port = port;
                }
                if let Some(name) = section.name {
                    config.name = name;
                }
            }

            if let Some(runtime) = toml.runtime {
                if let Some(claude) = runtime.claude_binary {
                    config.runtime.claude_binary = claude;
                }
                if let Some(codex) = runtime.codex_binary {
                    config.runtime.codex_binary = codex;
                }
            }
        }

        // CLI overrides
        if let Some(port) = cli_port {
            config.port = port;
        }
        if let Some(bind) = cli_bind {
            config.bind = bind.to_string();
        }
        if let Some(workspace) = cli_workspace {
            config.workspace_root = PathBuf::from(workspace);
        }

        // Validate
        if !config.workspace_root.exists() {
            // Will be created during workspace initialization
            log::warn!(
                "[BridgeConfig] Workspace root does not exist: {} (will be created)",
                config.workspace_root.display()
            );
        }

        Ok(config)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BridgeConfig::default();
        assert_eq!(config.bind, "0.0.0.0");
        assert_eq!(config.port, 7878);
        assert!(config.workspace_root.to_string_lossy().contains(".agentszone"));
    }

    #[test]
    fn test_resolve_with_cli_overrides() {
        let dir = tempfile::tempdir().unwrap();
        let config = BridgeConfig::resolve(
            Some(9090),
            Some("127.0.0.1"),
            None,
            Some(dir.path().to_str().unwrap()),
        )
        .unwrap();
        assert_eq!(config.port, 9090);
        assert_eq!(config.bind, "127.0.0.1");
        assert_eq!(config.workspace_root, dir.path());
    }

    #[test]
    fn test_resolve_with_toml_config() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[bridge]
port = 8080
bind = "192.168.1.1"
name = "test-bridge"
"#;
        let config_path = dir.path().join("bridge.toml");
        std::fs::write(&config_path, toml_content).unwrap();

        let config = BridgeConfig::resolve(
            None,
            None,
            Some(config_path.to_str().unwrap()),
            None,
        )
        .unwrap();
        assert_eq!(config.port, 8080);
        assert_eq!(config.bind, "192.168.1.1");
        assert_eq!(config.name, "test-bridge");
    }

    #[test]
    fn test_cli_overrides_toml() {
        let dir = tempfile::tempdir().unwrap();
        let toml_content = r#"
[bridge]
port = 8080
"#;
        let config_path = dir.path().join("bridge.toml");
        std::fs::write(&config_path, toml_content).unwrap();

        let config = BridgeConfig::resolve(
            Some(9999),
            None,
            Some(config_path.to_str().unwrap()),
            None,
        )
        .unwrap();
        // CLI port overrides TOML
        assert_eq!(config.port, 9999);
    }

    #[test]
    fn test_missing_config_file_uses_defaults() {
        let config = BridgeConfig::resolve(
            None,
            None,
            Some("/nonexistent/bridge.toml"),
            None,
        )
        .unwrap();
        assert_eq!(config.port, 7878);
        assert_eq!(config.bind, "0.0.0.0");
    }
}
