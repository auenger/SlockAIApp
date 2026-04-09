export type TabType = 'CHAT' | 'TASKS' | 'WORKSPACE' | 'SKILLS' | 'ACTIVITY' | 'PROFILE';

/** Message role in a conversation */
export type MessageRole = "user" | "agent" | "system";

/** Agent identifier */
export type AgentId = string;

export interface Agent {
  id: string;
  name: string;
  description: string;
  status: 'online' | 'offline' | 'busy';
  avatar: string;
  color: string;
}

export interface Channel {
  id: string;
  name: string;
  unreadCount?: number;
}

export interface Thread {
  id: string;
  agent_id: string;
  title: string;
  session_id: string | null;
  messages: ThreadMessageData[];
  created_at: string;
  updated_at: string;
}

export interface ThreadMessageData {
  id: string;
  role: "user" | "agent";
  content: string;
  timestamp: string;
}

export interface ThreadInfo {
  id: string;
  agent_id: string;
  title: string;
  preview: string;
  message_count: number;
  created_at: string;
  updated_at: string;
}

export interface Task {
  id: number;
  title: string;
  status: 'TODO' | 'IN PROGRESS' | 'IN REVIEW' | 'DONE';
  assignee?: string;
}

export interface Message {
  id: string;
  sender: {
    name: string;
    avatar: string;
    isAgent: boolean;
  };
  content: string;
  timestamp: string;
  isThinking?: boolean;
}

// ===========================================================================
// Agent Runtime types
// ===========================================================================

/** Agent runtime status */
export type AgentRuntimeStatusType =
  | "available"
  | "unhealthy"
  | "not-installed"
  | "detecting";

/** Agent runtime capability */
export type AgentCapability =
  | "streaming"
  | "sessions"
  | "tool_use"
  | "structured_output";

/** Information about a registered agent runtime */
export interface AgentRuntimeInfo {
  id: string;
  name: string;
  runtime_type: string;
  status: AgentRuntimeStatusType;
  version?: string;
  install_path?: string;
  capabilities: AgentCapability[];
  install_hint: string;
}

/** Streaming event from an agent runtime execution */
export interface StreamEvent {
  text: string;
  is_done: boolean;
  error?: string;
  type?: string;
  session_id?: string;
}

// ===========================================================================
// Agent Workspace types
// ===========================================================================

/** Agent summary returned from the backend */
export interface AgentSummary {
  agent_id: string;
  name: string;
  emoji: string;
  avatar: string | null;
  enabled: boolean;
  session_count: number;
}

/** Agent identity metadata */
export interface IdentitySummary {
  agent_id: string;
  name: string;
  emoji: string;
  avatar: string | null;
  creature: string;
  vibe: string;
}

/** Request to create a new Agent */
export interface CreateAgentRequest {
  name: string;
  creature?: string;
  vibe?: string;
  emoji?: string;
  avatar?: string;
}

/** Workspace and Agent manager status */
export interface ManagerStatus {
  total_agents: number;
  enabled_agents: number;
  active_agent_id: string | null;
  workspace_root: string;
}

/** Result of workspace initialization */
export interface InitWorkspaceResult {
  templates_created: string[];
  templates_skipped: string[];
  default_created: boolean;
}

/** Agent context information (for debugging/preview) */
export interface AgentContextResult {
  agent_id: string;
  system_prompt: string;
  has_user_context: boolean;
  has_agent_instructions: boolean;
  has_tool_instructions: boolean;
  has_memory: boolean;
  has_history: boolean;
  context_prefix_length: number;
}

// ===========================================================================
// Agent with Runtime status types
// ===========================================================================

/** Agent summary fused with its runtime availability status */
export interface AgentWithRuntime {
  /** Agent workspace summary */
  agent: AgentSummary;
  /** Runtime status: "available" | "not-installed" | "unhealthy" | "detecting" */
  runtime_status: AgentRuntimeStatusType;
  /** Detected runtime version (if available) */
  runtime_version?: string;
  /** Install hint (shown when runtime not installed) */
  runtime_install_hint?: string;
}
