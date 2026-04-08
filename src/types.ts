/**
 * Base type definitions for SlockAI.
 */

/** Message role in a conversation */
export type MessageRole = "user" | "agent" | "system";

/** Agent identifier */
export type AgentId = "claude" | "codex";

/** A single message in a channel */
export interface Message {
  id: string;
  role: MessageRole;
  content: string;
  timestamp: string;
  channel: string;
  agent?: AgentId;
}

/** A channel (conversation container) */
export interface Channel {
  id: string;
  name: string;
  createdAt: string;
}

/** Agent status */
export type AgentStatus = "idle" | "running" | "error";

/** Agent info */
export interface Agent {
  id: AgentId;
  name: string;
  status: AgentStatus;
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
