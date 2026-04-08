export type TabType = 'CHAT' | 'TASKS' | 'WORKSPACE' | 'SKILLS' | 'ACTIVITY' | 'PROFILE';

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
  title: string;
  preview: string;
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
