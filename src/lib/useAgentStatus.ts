/**
 * Hook for managing agent status in AgentsZone.
 *
 * Combines workspace agent info with runtime availability status.
 * Provides a unified interface for the Sidebar to display agents
 * with real status indicators and for the App to track agent selection.
 */

import { useState, useCallback, useEffect } from "react";
import type { AgentWithRuntime, AgentRuntimeStatusType } from "../types";
import { getAgentRuntimeStatus, scanAgentRuntimes } from "./ipc";

// ---------------------------------------------------------------------------
// Dev fallback: mock data when not running inside Tauri
// ---------------------------------------------------------------------------

const isTauri = "__TAURI_INTERNALS__" in window;

const MOCK_AGENTS: AgentWithRuntime[] = [
  {
    agent: {
      agent_id: "default",
      name: "AgentsZone",
      emoji: "robot",
      avatar: null,
      enabled: true,
      session_count: 0,
    },
    runtime_status: "available",
    runtime_version: "1.0.0-dev",
    runtime_install_hint: "npm install -g @anthropic-ai/claude-code",
  },
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Map runtime status to a display color. */
export function getRuntimeStatusColor(status: AgentRuntimeStatusType): string {
  switch (status) {
    case "available":
      return "#39FF14"; // green
    case "unhealthy":
      return "#FFDE00"; // yellow
    case "not-installed":
      return "#9CA3AF"; // gray
    case "detecting":
      return "#00E5FF"; // cyan
    default:
      return "#9CA3AF";
  }
}

/** Map runtime status to a display label. */
export function getRuntimeStatusLabel(status: AgentRuntimeStatusType): string {
  switch (status) {
    case "available":
      return "Online";
    case "unhealthy":
      return "Unhealthy";
    case "not-installed":
      return "Not Installed";
    case "detecting":
      return "Detecting...";
    default:
      return "Unknown";
  }
}

// ---------------------------------------------------------------------------
// Hook return type
// ---------------------------------------------------------------------------

export interface AgentStatusState {
  /** List of agents with their runtime status */
  agents: AgentWithRuntime[];
  /** Whether a scan is in progress */
  loading: boolean;
  /** Trigger a runtime rescan and reload agent status */
  scan: () => Promise<void>;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useAgentStatus(): AgentStatusState {
  const [agents, setAgents] = useState<AgentWithRuntime[]>([]);
  const [loading, setLoading] = useState(false);

  /** Scan runtimes and fetch agent status */
  const scan = useCallback(async () => {
    setLoading(true);
    try {
      if (!isTauri) {
        setAgents(MOCK_AGENTS);
        return;
      }

      // First, trigger a runtime scan to refresh detection data
      await scanAgentRuntimes();

      // Then, fetch the fused agent + runtime status
      const result = await getAgentRuntimeStatus();
      setAgents(result);
    } catch (err) {
      console.error("[useAgentStatus] scan failed:", err);
      if (!isTauri) {
        setAgents(MOCK_AGENTS);
      }
    } finally {
      setLoading(false);
    }
  }, []);

  // Auto-scan on mount
  useEffect(() => {
    scan();
  }, [scan]);

  return { agents, loading, scan };
}
