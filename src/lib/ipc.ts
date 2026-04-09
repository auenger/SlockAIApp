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
