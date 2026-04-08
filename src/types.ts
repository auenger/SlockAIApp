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
