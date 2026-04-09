//! Agent Workspace module.
//!
//! Provides isolated file directories per Agent, template initialization,
//! and workspace management for the AgentsZone multi-Agent system.
//!
//! ## Directory Layout
//!
//! ```text
//! workspaces/
//! ├── SOUL.md              # Global Agent personality (default)
//! ├── USER.md              # User profile
//! ├── AGENTS.md            # Agent behavior instructions
//! ├── TOOLS.md             # Tool usage guide
//! ├── memory/              # Memory storage
//! │   ├── MEMORY.md        # Long-term memory
//! │   └── HISTORY.md       # History summary
//! └── agents/              # Multi-Agent directory
//!     ├── default/         # Default Agent
//!     │   ├── IDENTITY.md  # Identity metadata
//!     │   ├── SOUL.md      # Personalized personality (overrides global)
//!     │   ├── conversations/  # Conversation records (JSONL)
//!     │   ├── context/        # Context snapshots
//!     │   ├── output/         # Agent output
//!     │   ├── skills/         # Agent skills
//!     │   └── config/         # Agent configuration
//!     └── <agent-name>/   # Custom Agents
//!         └── ...
//! ```

pub mod agent;
pub mod channel;
pub mod identity;
pub mod manager;
pub mod mention;
pub mod templates;
pub mod thread;

pub use agent::AgentWorkspace;
pub use channel::{Channel, ChannelInfo, ChannelMember, ChannelMessage, ChannelStore};
pub use identity::AgentIdentity;
pub use manager::AgentManager;
pub use mention::{Mention, MentionResult, parse_mentions, resolve_agents};
pub use templates::WorkspaceTemplates;
pub use thread::{Thread, ThreadInfo, ThreadMessage, ThreadStore};
