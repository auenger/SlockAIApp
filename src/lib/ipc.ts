/**
 * Type-safe Tauri IPC wrapper.
 *
 * Provides invoke and listen helpers with proper TypeScript typing
 * for communication between the React frontend and Rust backend.
 */

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import type {
  AgentSummary,
  CreateAgentRequest,
  ManagerStatus,
  InitWorkspaceResult,
  IdentitySummary,
  AgentContextResult,
  AgentWithRuntime,
  AgentRuntimeInfo,
  Thread,
  ThreadInfo,
  Channel,
  ChannelInfo,
} from "../types";

/**
 * Type-safe invoke wrapper for Tauri commands.
 */
export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return tauriInvoke<T>(cmd, args);
}

// ---------------------------------------------------------------------------
// Test command
// ---------------------------------------------------------------------------

/** Send a greeting to the Rust backend (test command). */
export async function greet(name: string): Promise<string> {
  return invoke<string>("greet", { name });
}

// ---------------------------------------------------------------------------
// Runtime commands
// ---------------------------------------------------------------------------

/** Scan for available agent runtimes. */
export async function scanAgentRuntimes(): Promise<AgentRuntimeInfo[]> {
  return invoke<AgentRuntimeInfo[]>("scan_agent_runtimes");
}

// ---------------------------------------------------------------------------
// Workspace commands
// ---------------------------------------------------------------------------

/** Initialize the workspace for the first time. */
export async function initWorkspace(): Promise<InitWorkspaceResult> {
  return invoke<InitWorkspaceResult>("init_workspace");
}

/** Get workspace and agent manager status. */
export async function getWorkspaceStatus(): Promise<ManagerStatus> {
  return invoke<ManagerStatus>("get_workspace_status");
}

// ---------------------------------------------------------------------------
// Agent CRUD commands
// ---------------------------------------------------------------------------

/** Create a new Agent. */
export async function createAgent(request: CreateAgentRequest): Promise<AgentSummary> {
  return invoke<AgentSummary>("create_agent", { request });
}

/** List all available Agents. */
export async function listAgents(): Promise<AgentSummary[]> {
  return invoke<AgentSummary[]>("list_agents");
}

/** Switch to a different Agent. */
export async function switchAgent(agentId: string): Promise<AgentSummary> {
  return invoke<AgentSummary>("switch_agent", { agentId });
}

/** Get the currently active Agent. */
export async function getActiveAgent(): Promise<AgentSummary | null> {
  return invoke<AgentSummary | null>("get_active_agent");
}

/** Delete an Agent by ID. */
export async function deleteAgent(agentId: string): Promise<void> {
  return invoke<void>("delete_agent", { agentId });
}

// ---------------------------------------------------------------------------
// Identity commands
// ---------------------------------------------------------------------------

/** Get the identity of a specific Agent. */
export async function getAgentIdentity(agentId: string): Promise<IdentitySummary> {
  return invoke<IdentitySummary>("get_agent_identity", { agentId });
}

// ---------------------------------------------------------------------------
// Context commands
// ---------------------------------------------------------------------------

/** Get the context information for an Agent. */
export async function getAgentContext(agentId: string): Promise<AgentContextResult> {
  return invoke<AgentContextResult>("get_agent_context", { agentId });
}

// ---------------------------------------------------------------------------
// Agent + Runtime status commands
// ---------------------------------------------------------------------------

/** Get all agents fused with their runtime availability status. */
export async function getAgentRuntimeStatus(): Promise<AgentWithRuntime[]> {
  return invoke<AgentWithRuntime[]>("get_agent_runtime_status");
}

// ---------------------------------------------------------------------------
// Thread commands
// ---------------------------------------------------------------------------

/** Create a new Thread for a specific agent. */
export async function createThread(agentId: string): Promise<Thread> {
  return invoke<Thread>("create_thread", { agentId });
}

/** List all threads for a specific agent. */
export async function listThreads(agentId: string): Promise<ThreadInfo[]> {
  return invoke<ThreadInfo[]>("list_threads", { agentId });
}

/** Get a single thread by ID. */
export async function getThread(agentId: string, threadId: string): Promise<Thread> {
  return invoke<Thread>("get_thread", { agentId, threadId });
}

/** Delete a thread by ID. */
export async function deleteThread(agentId: string, threadId: string): Promise<void> {
  return invoke<void>("delete_thread", { agentId, threadId });
}

/** Send a message in a thread (triggers runtime streaming). */
export async function sendMessage(
  agentId: string,
  threadId: string,
  message: string
): Promise<Thread> {
  return invoke<Thread>("send_message", { agentId, threadId, message });
}

/** Save an agent response to a thread (after streaming completes). */
export async function saveAgentResponse(
  agentId: string,
  threadId: string,
  content: string,
  sessionId: string | null
): Promise<Thread> {
  return invoke<Thread>("save_agent_response", { agentId, threadId, content, sessionId });
}

// ---------------------------------------------------------------------------
// Channel commands
// ---------------------------------------------------------------------------

/** Create a new Channel with the given name and Agent members. */
export async function createChannel(
  name: string,
  memberAgentIds: string[]
): Promise<Channel> {
  return invoke<Channel>("create_channel", {
    request: { name, member_agent_ids: memberAgentIds },
  });
}

/** List all channels. */
export async function listChannels(): Promise<ChannelInfo[]> {
  return invoke<ChannelInfo[]>("list_channels");
}

/** Get a single channel by ID (with full details). */
export async function getChannel(channelId: string): Promise<Channel> {
  return invoke<Channel>("get_channel", { channelId });
}

/** Update a channel's settings. */
export async function updateChannel(
  channelId: string,
  name?: string
): Promise<Channel> {
  return invoke<Channel>("update_channel", { channelId, request: { name } });
}

/** Delete a channel by ID. */
export async function deleteChannel(channelId: string): Promise<void> {
  return invoke<void>("delete_channel", { channelId });
}

/** Add an Agent member to a channel. */
export async function addChannelMember(
  channelId: string,
  agentId: string
): Promise<Channel> {
  return invoke<Channel>("add_channel_member", { channelId, agentId });
}

/** Remove an Agent member from a channel. */
export async function removeChannelMember(
  channelId: string,
  agentId: string
): Promise<Channel> {
  return invoke<Channel>("remove_channel_member", { channelId, agentId });
}

/** Send a message in a channel (triggers runtime streaming). */
export async function sendChannelMessage(
  channelId: string,
  message: string
): Promise<Channel> {
  return invoke<Channel>("send_channel_message", { channelId, message });
}

/** Save an agent response to a channel (after streaming completes). */
export async function saveChannelResponse(
  channelId: string,
  agentId: string,
  content: string
): Promise<Channel> {
  return invoke<Channel>("save_channel_response", { channelId, agentId, content });
}
