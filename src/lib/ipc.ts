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
  UpdateAgentRequest,
  ManagerStatus,
  InitWorkspaceResult,
  IdentitySummary,
  AgentContextResult,
  AgentWithRuntime,
  AgentRuntimeInfo,
  RuntimeType,
  Thread,
  ThreadInfo,
  ThreadMessageData,
  Channel,
  ChannelInfo,
  DirectoryEntry,
  FileContent,
  ApiKeyInfo,
  SkillInfo,
  ActivityLogEntry,
  ListActivitiesResult,
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

/** List all registered runtimes using cached detection data. */
export async function listAgentRuntimes(): Promise<AgentRuntimeInfo[]> {
  return invoke<AgentRuntimeInfo[]>("list_agent_runtimes");
}

/** Get detailed info about a specific runtime by type. */
export async function getRuntimeInfo(runtimeType: RuntimeType): Promise<AgentRuntimeInfo> {
  return invoke<AgentRuntimeInfo>("get_runtime_info", { runtimeType });
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

/** Health check result for the workspace. */
export interface HealthCheckResult {
  agents: import("../types").AgentHealthInfo[];
  repaired: number;
  still_unhealthy: number;
}

/** Perform a workspace health check and repair. */
export async function healthCheckWorkspace(): Promise<HealthCheckResult> {
  return invoke<HealthCheckResult>("health_check_workspace");
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

/** Update an existing Agent's mutable properties. */
export async function updateAgent(
  agentId: string,
  request: UpdateAgentRequest
): Promise<AgentSummary> {
  return invoke<AgentSummary>("update_agent", { agentId, request });
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
// Workspace browsing commands
// ---------------------------------------------------------------------------

/** List directory entries in an agent's workspace. */
export async function listWorkspaceDir(
  agentId: string,
  subpath?: string
): Promise<DirectoryEntry[]> {
  return invoke<DirectoryEntry[]>("list_workspace_dir", { agentId, subpath: subpath ?? null });
}

/** Read the content of a file from an agent's workspace. */
export async function readWorkspaceFile(
  agentId: string,
  filePath: string
): Promise<FileContent> {
  return invoke<FileContent>("read_workspace_file", { agentId, filePath });
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

/** Load messages for a thread from JSONL storage (for crash recovery). */
export async function loadThreadMessages(
  agentId: string,
  threadId: string,
  limit?: number
): Promise<ThreadMessageData[]> {
  return invoke<ThreadMessageData[]>("load_thread_messages", { agentId, threadId, limit: limit ?? null });
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

/** Compact (summarize) older messages in a channel. */
export async function compactChannel(
  channelId: string
): Promise<Channel> {
  return invoke<Channel>("compact_channel", { channelId });
}

// ---------------------------------------------------------------------------
// API Key management commands
// ---------------------------------------------------------------------------

/** List all API keys (masked) for known providers. */
export async function listApiKeys(): Promise<ApiKeyInfo[]> {
  return invoke<ApiKeyInfo[]>("list_api_keys");
}

/** Store a new API key for a provider. */
export async function storeApiKey(runtimeId: string, apiKey: string): Promise<void> {
  return invoke<void>("store_api_key", { runtimeId, apiKey });
}

/** Check if an API key exists for a provider. */
export async function hasApiKey(runtimeId: string): Promise<boolean> {
  return invoke<boolean>("has_api_key", { runtimeId });
}

/** Delete an API key for a provider. */
export async function deleteApiKey(runtimeId: string): Promise<void> {
  return invoke<void>("delete_api_key", { runtimeId });
}

/** Verify an API key and return its masked info. */
export async function verifyApiKey(runtimeId: string): Promise<ApiKeyInfo> {
  return invoke<ApiKeyInfo>("verify_api_key", { runtimeId });
}

// ---------------------------------------------------------------------------
// Skill management commands
// ---------------------------------------------------------------------------

/** List all Skills for a given Agent. */
export async function listSkills(agentId: string): Promise<SkillInfo[]> {
  return invoke<SkillInfo[]>("list_skills", { agentId });
}

/** Add a new Skill to an Agent. */
export async function addSkill(
  agentId: string,
  name: string,
  skillType: string,
  config: Record<string, unknown>
): Promise<SkillInfo> {
  return invoke<SkillInfo>("add_skill", {
    agentId,
    request: { name, skill_type: skillType, config },
  });
}

/** Update an existing Skill. */
export async function updateSkill(
  agentId: string,
  skillId: string,
  request: {
    name?: string;
    skill_type?: string;
    config?: Record<string, unknown>;
    status?: string;
  }
): Promise<SkillInfo> {
  return invoke<SkillInfo>("update_skill", { agentId, skillId, request });
}

/** Delete a Skill. */
export async function deleteSkill(
  agentId: string,
  skillId: string
): Promise<void> {
  return invoke<void>("delete_skill", { agentId, skillId });
}

/** Get the status of a single Skill. */
export async function getSkillStatus(
  agentId: string,
  skillId: string
): Promise<SkillInfo> {
  return invoke<SkillInfo>("get_skill_status", { agentId, skillId });
}

// ---------------------------------------------------------------------------
// Activity Log commands
// ---------------------------------------------------------------------------

/** Log a new activity entry. */
export async function logActivity(params: {
  activity_type: string;
  agent_id?: string | null;
  summary: string;
  details?: Record<string, unknown>;
}): Promise<ActivityLogEntry> {
  return invoke<ActivityLogEntry>("log_activity", {
    request: {
      activity_type: params.activity_type,
      agent_id: params.agent_id ?? null,
      summary: params.summary,
      details: params.details ?? {},
    },
  });
}

/** List activity entries with optional filter and pagination. */
export async function listActivities(params?: {
  agent_id?: string | null;
  offset?: number;
  limit?: number;
}): Promise<ListActivitiesResult> {
  return invoke<ListActivitiesResult>("list_activities", {
    request: {
      agent_id: params?.agent_id ?? null,
      offset: params?.offset ?? 0,
      limit: params?.limit ?? 50,
    },
  });
}

/** Clear all activity entries. */
export async function clearActivities(): Promise<void> {
  return invoke<void>("clear_activities");
}
