/**
 * Hook for loading Agent profile data for the Profile tab.
 *
 * Fetches identity, context (role), and workspace information.
 */

import { useState, useCallback } from "react";
import type { IdentitySummary, AgentContextResult, ManagerStatus } from "../types";
import { getAgentIdentity, getAgentContext, getWorkspaceStatus } from "./ipc";

// ---------------------------------------------------------------------------
// Dev fallback: mock data when not running inside Tauri
// ---------------------------------------------------------------------------

const isTauri = "__TAURI_INTERNALS__" in window;

const MOCK_IDENTITY: IdentitySummary = {
  agent_id: "default",
  name: "克劳德",
  emoji: "🤖",
  avatar: null,
  creature: "AI",
  vibe: "专业",
};

const MOCK_CONTEXT: AgentContextResult = {
  agent_id: "default",
  system_prompt: "你是一个非常资深的软件开发工程师，负责软件的架构设计和开发",
  has_user_context: false,
  has_agent_instructions: true,
  has_tool_instructions: true,
  has_memory: false,
  has_history: false,
  context_prefix_length: 0,
};

const MOCK_WORKSPACE: ManagerStatus = {
  total_agents: 1,
  enabled_agents: 1,
  active_agent_id: "default",
  workspace_root: "/Users/ryan/AgentsZone/workspaces/default",
};

// ---------------------------------------------------------------------------
// Hook return type
// ---------------------------------------------------------------------------

export interface AgentProfileData {
  identity: IdentitySummary | null;
  context: AgentContextResult | null;
  workspace: ManagerStatus | null;
}

export interface AgentProfileState {
  data: AgentProfileData;
  loading: boolean;
  error: string | null;
  /** Reload profile data for a specific agent */
  loadProfile: (agentId: string) => Promise<void>;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useAgentProfile(): AgentProfileState {
  const [data, setData] = useState<AgentProfileData>({
    identity: null,
    context: null,
    workspace: null,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /** Load profile data for a specific agent */
  const loadProfile = useCallback(async (agentId: string) => {
    if (!agentId) {
      setData({ identity: null, context: null, workspace: null });
      return;
    }

    setLoading(true);
    setError(null);

    try {
      // Fetch all data in parallel
      const [identity, context, workspace] = await Promise.all([
        isTauri ? getAgentIdentity(agentId) : Promise.resolve(MOCK_IDENTITY),
        isTauri ? getAgentContext(agentId) : Promise.resolve(MOCK_CONTEXT),
        isTauri ? getWorkspaceStatus() : Promise.resolve(MOCK_WORKSPACE),
      ]);

      setData({ identity, context, workspace });
    } catch (err) {
      console.error("[useAgentProfile] load failed:", err);
      setError(err instanceof Error ? err.message : "Failed to load profile");
      // Set mock data on error for dev
      if (!isTauri) {
        setData({
          identity: MOCK_IDENTITY,
          context: MOCK_CONTEXT,
          workspace: MOCK_WORKSPACE,
        });
      }
    } finally {
      setLoading(false);
    }
  }, []);

  return { data, loading, error, loadProfile };
}
